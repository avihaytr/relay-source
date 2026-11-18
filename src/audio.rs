// SPDX-License-Identifier: MPL-2.0
//! Native playback of 48 kHz stereo PCM with a hard 80 ms queue limit.
use std::{collections::VecDeque, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}}};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const SOURCE_RATE: f64 = 48_000.;
const MAX_FRAMES: usize = 3_840;
const START_FRAMES: usize = 960;

#[derive(Default)]
pub struct AudioControl {
    pub muted: AtomicBool,
    pub available: AtomicBool,
    pub received_frames: AtomicU64,
    pub audible_frames: AtomicU64,
    pub dropped_frames: AtomicU64,
    pub error: Mutex<Option<String>>,
}
impl AudioControl {
    pub fn toggle(&self) { self.muted.fetch_xor(true, Ordering::Relaxed); }
}

struct PcmQueue {
    frames: VecDeque<[f32; 2]>,
    phase: f64,
    primed: bool,
}
impl PcmQueue {
    fn new() -> Self { Self {frames:VecDeque::with_capacity(MAX_FRAMES), phase:0., primed:false} }
    fn push(&mut self, pcm: &[u8]) -> u64 {
        let mut dropped=0;
        for bytes in pcm.chunks_exact(4) {
            if self.frames.len()==MAX_FRAMES {self.frames.pop_front();dropped+=1;}
            self.frames.push_back([
                i16::from_le_bytes([bytes[0],bytes[1]]) as f32/32768.,
                i16::from_le_bytes([bytes[2],bytes[3]]) as f32/32768.,
            ]);
        }
        dropped
    }
    fn next(&mut self, output_rate:u32) -> [f32;2] {
        if !self.primed {
            if self.frames.len()<START_FRAMES{return [0.;2];}
            self.primed=true;
        }
        if self.frames.len()<2 {
            self.frames.clear();self.primed=false;self.phase=0.;return [0.;2];
        }
        let a=self.frames[0];let b=self.frames[1];let t=self.phase as f32;
        let value=[a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t];
        self.phase+=SOURCE_RATE/output_rate as f64;
        while self.phase>=1. {self.frames.pop_front();self.phase-=1.;}
        value
    }
}

pub struct AudioOutput {
    _stream:cpal::Stream,
    queue:Arc<Mutex<PcmQueue>>,
    control:Arc<AudioControl>,
}
impl AudioOutput {
    pub fn open(control:Arc<AudioControl>) -> Result<Self,String> {
        let device=cpal::default_host().default_output_device().ok_or("No audio output device is available")?;
        let mut choices:Vec<_>=device.supported_output_configs().map_err(|e|e.to_string())?
            .filter(|c|c.channels()==2&&c.min_sample_rate().0<=48_000&&c.max_sample_rate().0>=48_000)
            .filter(|c|matches!(c.sample_format(),cpal::SampleFormat::F32|cpal::SampleFormat::I16|cpal::SampleFormat::U16)).collect();
        choices.sort_by_key(|c|if c.sample_format()==cpal::SampleFormat::F32{0}else{1});
        let supported=if let Some(config)=choices.into_iter().next(){config.with_sample_rate(cpal::SampleRate(48_000))}else{device.default_output_config().map_err(|e|e.to_string())?};
        let mut config=supported.config();if let cpal::SupportedBufferSize::Range{min,max}=supported.buffer_size(){config.buffer_size=cpal::BufferSize::Fixed(1024u32.clamp(*min,*max));}let queue=Arc::new(Mutex::new(PcmQueue::new()));
        let stream=match supported.sample_format() {
            cpal::SampleFormat::F32=>build::<f32>(&device,&config,queue.clone(),control.clone()),
            cpal::SampleFormat::I16=>build::<i16>(&device,&config,queue.clone(),control.clone()),
            cpal::SampleFormat::U16=>build::<u16>(&device,&config,queue.clone(),control.clone()),
            format=>Err(format!("Unsupported audio output format: {format:?}")),
        }?;
        stream.play().map_err(|e|e.to_string())?;
        control.available.store(true,Ordering::Relaxed);
        eprintln!("Relay: device audio output ready · {} Hz · {} channels",config.sample_rate.0,config.channels);
        Ok(Self {_stream:stream,queue,control})
    }
    pub fn push(&mut self, pcm:&[u8]) {
        let dropped=self.queue.lock().unwrap().push(pcm);
        self.control.dropped_frames.fetch_add(dropped,Ordering::Relaxed);
        if self.control.received_frames.fetch_add((pcm.len()/4)as u64,Ordering::Relaxed)==0 {
            eprintln!("Relay: receiving encrypted device audio · 48000 Hz stereo PCM");
        }
    }
}
impl Drop for AudioOutput {
    fn drop(&mut self) {
        self.control.available.store(false,Ordering::Relaxed);
        eprintln!("Relay: audio stopped · received {} frames · played {} audible frames · dropped {} stale frames",
            self.control.received_frames.load(Ordering::Relaxed),self.control.audible_frames.load(Ordering::Relaxed),self.control.dropped_frames.load(Ordering::Relaxed));
    }
}
fn build<T:cpal::SizedSample+cpal::FromSample<f32>>(
    device:&cpal::Device, config:&cpal::StreamConfig,queue:Arc<Mutex<PcmQueue>>,control:Arc<AudioControl>
) -> Result<cpal::Stream,String> {
    let channels=config.channels as usize;let rate=config.sample_rate.0;
    let errors=control.clone();
    device.build_output_stream(config,move|output:&mut[T],_|{
        let muted=control.muted.load(Ordering::Relaxed);let mut audible=0u64;
        if let Ok(mut queue)=queue.try_lock() {
            for frame in output.chunks_mut(channels) {
                let sample=queue.next(rate);
                if !muted&&(sample[0].abs()>0.0005||sample[1].abs()>0.0005){audible+=1;}
                for (channel,dest) in frame.iter_mut().enumerate(){
                    let value=if muted{0.}else if channels==1{(sample[0]+sample[1])*0.5}else if channel<2{sample[channel]}else{0.};
                    *dest=T::from_sample(value);
                }
            }
        } else {for dest in output {*dest=T::from_sample(0.);}}
        control.audible_frames.fetch_add(audible,Ordering::Relaxed);
    },move|error|{errors.available.store(false,Ordering::Relaxed);*errors.error.lock().unwrap()=Some(error.to_string());eprintln!("Relay: audio output error: {error}");},None).map_err(|e|e.to_string())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn congestion_drops_old_audio_instead_of_growing_latency() {
        let mut queue=PcmQueue::new();
        assert_eq!(queue.push(&vec![0;MAX_FRAMES*4]),0);
        assert_eq!(queue.push(&[0xff,0x7f,0,0]),1);
        assert_eq!(queue.frames.len(),MAX_FRAMES);
        assert!(queue.frames.back().unwrap()[0]>0.99);
    }
    #[test] fn stereo_channels_are_preserved_and_underruns_go_silent() {
        let mut queue=PcmQueue::new();let mut data=vec![];
        for _ in 0..START_FRAMES {data.extend_from_slice(&[0,0x40,0,0xc0]);}
        queue.push(&data);assert_eq!(queue.next(48_000),[0.5,-0.5]);
        for _ in 0..START_FRAMES {queue.next(48_000);}
        assert_eq!(queue.next(48_000),[0.,0.]);assert!(!queue.primed);
    }
    #[test] fn rate_conversion_interpolates_samples() {
        let mut queue=PcmQueue::new();queue.primed=true;
        queue.frames.extend([[0.,0.],[1.,-1.],[0.,0.]]);
        assert_eq!(queue.next(96_000),[0.,0.]);
        assert_eq!(queue.next(96_000),[0.5,-0.5]);
        assert_eq!(queue.next(96_000),[1.,-1.]);
    }
}
