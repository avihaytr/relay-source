// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;
import android.app.*;
import android.os.*;
import android.content.*;
import android.content.pm.ServiceInfo;
import android.hardware.display.*;
import android.media.*;
import android.media.projection.*;
import android.net.Uri;
import android.view.Surface;
import java.io.*;
import java.nio.ByteBuffer;
import javax.net.ssl.SSLSocket;

public class ShareService extends Service {
    public static volatile boolean sharing,connecting,audioAvailable,muted;
    public static volatile String receiverName="Your computer",audioStatus="Screen only",lastError="";
    private boolean shareAudio;
    private int quality=2;
    private String deviceName="Android phone";
    private volatile AudioRecord playbackRecord;
    private volatile boolean audioStopped;
    public static void toggleMuted(){if(sharing&&audioAvailable)muted=!muted;}

    private volatile boolean stopped;
    private volatile SSLSocket socket;
    private MediaProjection projection;
    private Thread worker;
    private volatile long captureDimensions;
    private void captureSize(int width,int height){if(width>0&&height>0)captureDimensions=((long)width<<32)|(height&0xffffffffL);}
    private int[] encoderSize(long dimensions){int width=(int)(dimensions>>>32),height=(int)dimensions;int cap=quality==2?1920:1280;float scale=Math.min(1f,cap/(float)Math.max(width,height));return new int[]{Math.max(2,((int)(width*scale)/2)*2),Math.max(2,((int)(height*scale)/2)*2)};}
    private int supportedProfile(long dimensions){for(int profile=quality;profile>=0;profile--){quality=profile;int[] size=encoderSize(dimensions);int fps=profile==0?30:60;for(MediaCodecInfo info:new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()){if(!info.isEncoder())continue;boolean hardware=Build.VERSION.SDK_INT>=29?info.isHardwareAccelerated():!info.getName().toLowerCase().contains("google");if(!hardware)continue;for(String type:info.getSupportedTypes())if(type.equalsIgnoreCase("video/avc")){try{if(info.getCapabilitiesForType(type).getVideoCapabilities().areSizeAndRateSupported(size[0],size[1],fps))return profile;}catch(Exception ignored){}}}}return 0;}
    @Override public void onConfigurationChanged(android.content.res.Configuration config){super.onConfigurationChanged(config);android.util.DisplayMetrics metrics=new android.util.DisplayMetrics();((android.view.WindowManager)getSystemService(WINDOW_SERVICE)).getDefaultDisplay().getRealMetrics(metrics);captureSize(metrics.widthPixels,metrics.heightPixels);}
    @Override public IBinder onBind(Intent i){return null;}
    @Override public int onStartCommand(Intent i,int flags,int id){
        if(i==null||"stop".equals(i.getAction())){stopSelf();return START_NOT_STICKY;}
        if(worker!=null)return START_NOT_STICKY;
        connecting=true;sharing=false;audioAvailable=false;muted=false;lastError="";audioStatus="Screen only";receiverName=i.getStringExtra("receiverName");if(receiverName==null||receiverName.isEmpty())receiverName="Your computer";shareAudio=i.getBooleanExtra("shareAudio",false);quality=Math.max(0,Math.min(2,i.getIntExtra("quality",2)));deviceName=i.getStringExtra("deviceName");if(deviceName==null||deviceName.isEmpty())deviceName=Build.MODEL;
        NotificationManager nm=(NotificationManager)getSystemService(NOTIFICATION_SERVICE);
        nm.createNotificationChannel(new NotificationChannel("sharing","Screen sharing",NotificationManager.IMPORTANCE_LOW));
        PendingIntent stop=PendingIntent.getService(this,0,new Intent(this,ShareService.class).setAction("stop"),PendingIntent.FLAG_IMMUTABLE);
        Notification n=new Notification.Builder(this,"sharing").setSmallIcon(android.R.drawable.ic_menu_view).setContentTitle("Relay is sharing your screen").setContentText("Tap Stop to end sharing").setOngoing(true).addAction(new Notification.Action.Builder(null,"Stop",stop).build()).build();
        if(Build.VERSION.SDK_INT>=29)startForeground(1,n,ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION);else startForeground(1,n);
        MediaProjectionManager manager=(MediaProjectionManager)getSystemService(MEDIA_PROJECTION_SERVICE);
        projection=manager.getMediaProjection(i.getIntExtra("result",-1),(Intent)i.getParcelableExtra("data"));
        projection.registerCallback(new MediaProjection.Callback(){@Override public void onStop(){stopSelf();}@Override public void onCapturedContentResize(int width,int height){captureSize(width,height);}},new Handler(getMainLooper()));
        String link=i.getStringExtra("link"); worker=new Thread(()->stream(link),"relay-h264");worker.start();return START_NOT_STICKY;
    }
    private MediaCodec createEncoder(int w,int h)throws IOException{
            MediaFormat format=MediaFormat.createVideoFormat("video/avc",w,h);
            format.setInteger(MediaFormat.KEY_COLOR_FORMAT,MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface);
            int fps=quality==0?30:60,bitrate=quality==0?4000000:quality==1?8000000:16000000;
            format.setInteger(MediaFormat.KEY_BIT_RATE,bitrate);format.setInteger(MediaFormat.KEY_FRAME_RATE,fps);format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL,1);
            format.setInteger("max-bframes",0);format.setInteger(MediaFormat.KEY_BITRATE_MODE,MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR);format.setInteger("latency",0);format.setInteger("priority",0);format.setFloat("max-fps-to-encoder",fps);format.setLong("repeat-previous-frame-after",100000L);
            MediaCodecInfo chosen=null;for(MediaCodecInfo candidate:new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos()){if(!candidate.isEncoder())continue;boolean hardware=Build.VERSION.SDK_INT>=29?candidate.isHardwareAccelerated():!candidate.getName().toLowerCase().contains("google");if(!hardware)continue;for(String type:candidate.getSupportedTypes())if(type.equalsIgnoreCase("video/avc")){try{if(candidate.getCapabilitiesForType(type).getVideoCapabilities().areSizeAndRateSupported(w,h,fps)){chosen=candidate;break;}}catch(Exception ignored){}}if(chosen!=null)break;}
            MediaCodec codec=chosen!=null?MediaCodec.createByCodecName(chosen.getName()):MediaCodec.createEncoderByType("video/avc");android.util.Log.i("Relay","Encoder: "+codec.getName());codec.configure(format,null,null,MediaCodec.CONFIGURE_FLAG_ENCODE);return codec;
    }
    private void stream(String link){
        MediaCodec codec=null;VirtualDisplay display=null;Surface surface=null;
        try {
            RelayConnection.configure(this);Uri u=Uri.parse(link);quality=Math.min(quality,Math.max(0,Math.min(2,Integer.parseInt(u.getQueryParameter("q")==null?"2":u.getQueryParameter("q")))));
            Uri selected=u.buildUpon().clearQuery().appendQueryParameter("v","6").appendQueryParameter("q",String.valueOf(quality)).appendQueryParameter("remember",u.getQueryParameter("remember")==null?"0":u.getQueryParameter("remember")).appendQueryParameter("computer",u.getQueryParameter("computer")==null?"Computer":u.getQueryParameter("computer")).build();
            SSLSocket connection=RelayConnection.open(selected,"send",deviceName,pendingSocket->{socket=pendingSocket;if(stopped)try{pendingSocket.close();}catch(Exception ignored){}});socket=connection;if(stopped)return;
            DataOutputStream out=new DataOutputStream(connection.getOutputStream());
            android.util.DisplayMetrics metrics=new android.util.DisplayMetrics();
            ((android.view.WindowManager)getSystemService(WINDOW_SERVICE)).getDefaultDisplay().getRealMetrics(metrics);
            if(captureDimensions==0)captureSize(metrics.widthPixels,metrics.heightPixels);
            int requestedQuality=quality;quality=supportedProfile(captureDimensions);if(quality<requestedQuality)android.util.Log.w("Relay","Hardware encoder supports profile "+quality+" only; using it instead of "+requestedQuality);int[] initialSize=encoderSize(captureDimensions);int w=initialSize[0],h=initialSize[1];
            codec=createEncoder(w,h);surface=codec.createInputSurface();codec.start();android.util.Log.i("Relay","Encoder ready: "+w+"x"+h+" at "+(quality==0?30:60)+" FPS profile");
            display=projection.createVirtualDisplay("Relay",w,h,metrics.densityDpi,DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,surface,null,null);
            sharing=true;connecting=false;startPlaybackAudio(out);
            MediaCodec.BufferInfo info=new MediaCodec.BufferInfo();byte[] bytes=new byte[256*1024];
            byte[] config=new byte[0];boolean first=true;while(!stopped){
                int[] size=encoderSize(captureDimensions);
                if(size[0]!=w||size[1]!=h){
                    display.setSurface(null);codec.stop();codec.release();codec=null;surface.release();surface=null;
                    w=size[0];h=size[1];codec=createEncoder(w,h);surface=codec.createInputSurface();codec.start();
                    display.resize(w,h,metrics.densityDpi);display.setSurface(surface);config=new byte[0];first=true;
                    android.util.Log.i("Relay","Capture resized without a new sharing prompt: "+w+"x"+h);
                }
                int index=codec.dequeueOutputBuffer(info,100000);if(index>=0){try{ByteBuffer b=codec.getOutputBuffer(index);if(b!=null&&info.size>0){b.position(info.offset);b.limit(info.offset+info.size);if((info.flags&MediaCodec.BUFFER_FLAG_CODEC_CONFIG)!=0){config=new byte[b.remaining()];b.get(config);continue;}boolean keyframe=(info.flags&MediaCodec.BUFFER_FLAG_KEY_FRAME)!=0;if(first){android.util.Log.i("Relay","First encoded packet");first=false;}synchronized(out){out.writeInt(info.size+(keyframe?config.length:0));if(keyframe)out.write(config);while(b.hasRemaining()){int count=Math.min(b.remaining(),bytes.length);b.get(bytes,0,count);out.write(bytes,0,count);}}}}finally{codec.releaseOutputBuffer(index,false);}}}
        }catch(Exception e){if(!stopped){boolean wasSharing=sharing;lastError=wasSharing?"":"Could not connect: "+e.getMessage();android.util.Log.w("Relay","Screen-sharing session ended");if(!wasSharing)new Handler(getMainLooper()).post(()->android.widget.Toast.makeText(this,"Relay: "+e.getMessage(),1).show());}}
        finally{sharing=false;connecting=false;stopPlaybackAudio();try{if(socket!=null)socket.close();}catch(Exception ignored){}if(display!=null)display.release();if(codec!=null){try{codec.stop();}catch(Exception ignored){}codec.release();}if(surface!=null)surface.release();stopSelf();}
    }
    private void startPlaybackAudio(final DataOutputStream out){
        if(!shareAudio){audioStatus="Screen only · device audio off";return;}
        if(Build.VERSION.SDK_INT<29){audioStatus="Device audio needs Android 10 or later";return;}
        if(checkSelfPermission(android.Manifest.permission.RECORD_AUDIO)!=android.content.pm.PackageManager.PERMISSION_GRANTED){audioStatus="Screen only · audio permission not granted";return;}
        new Thread(()->{
            AudioRecord record=null;
            try{
                AudioPlaybackCaptureConfiguration capture=new AudioPlaybackCaptureConfiguration.Builder(projection)
                    .addMatchingUsage(AudioAttributes.USAGE_MEDIA).addMatchingUsage(AudioAttributes.USAGE_GAME).addMatchingUsage(AudioAttributes.USAGE_UNKNOWN).build();
                AudioFormat format=new AudioFormat.Builder().setEncoding(AudioFormat.ENCODING_PCM_16BIT).setSampleRate(48000).setChannelMask(AudioFormat.CHANNEL_IN_STEREO).build();
                int min=AudioRecord.getMinBufferSize(48000,AudioFormat.CHANNEL_IN_STEREO,AudioFormat.ENCODING_PCM_16BIT);
                record=new AudioRecord.Builder().setAudioFormat(format).setBufferSizeInBytes(Math.max(min,19200)).setAudioPlaybackCaptureConfig(capture).build();
                playbackRecord=record;if(stopped||audioStopped)return;
                record.startRecording();if(record.getRecordingState()!=AudioRecord.RECORDSTATE_RECORDING)throw new IOException("Playback capture did not start");
                audioAvailable=true;audioStatus="Device audio on";android.util.Log.i("Relay","Playback capture started · 48000 Hz stereo");
                short[] samples=new short[960];byte[] pcm=new byte[1920];boolean heardAudio=false;
                while(!stopped&&!audioStopped){
                    int count=record.read(samples,0,samples.length,AudioRecord.READ_BLOCKING);
                    if(count<0)throw new IOException("Playback capture read failed: "+count);
                    count-=count%2;if(count==0)continue;
                    boolean silent=muted;boolean nonzero=false;
                    for(int index=0;index<count;index++){short value=silent?0:samples[index];pcm[index*2]=(byte)value;pcm[index*2+1]=(byte)(value>>8);if(value>16||value< -16)nonzero=true;}
                    if(nonzero&&!heardAudio){heardAudio=true;android.util.Log.i("Relay","Device audio has non-silent samples");}
                    audioStatus=silent?"Shared audio muted":"Device audio on";
                    synchronized(out){out.writeInt(0x80000000|count*2);out.write(pcm,0,count*2);}
                }
            }catch(Exception error){if(!stopped&&!audioStopped){audioStatus="Device audio unavailable · screen sharing continues";android.util.Log.e("Relay","Playback capture failed",error);}}
            finally{audioAvailable=false;playbackRecord=null;if(record!=null){try{record.stop();}catch(Exception ignored){}record.release();}}
        },"relay-playback-audio").start();
    }
    private void stopPlaybackAudio(){audioStopped=true;AudioRecord record=playbackRecord;if(record!=null){try{record.stop();}catch(Exception ignored){}}}
    public static void playTestSound(){
        new Thread(()->{
            AudioTrack track=null;
            try{
                int frames=24000;short[] samples=new short[frames*2];
                for(int i=0;i<frames;i++){double envelope=Math.min(1.,Math.min(i/480.,(frames-i)/480.));short value=(short)(Math.sin(2.*Math.PI*440.*i/48000.)*6500.*envelope);samples[i*2]=samples[i*2+1]=value;}
                AudioAttributes.Builder attributes=new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC);
                if(Build.VERSION.SDK_INT>=29)attributes.setAllowedCapturePolicy(AudioAttributes.ALLOW_CAPTURE_BY_ALL);
                track=new AudioTrack.Builder().setAudioAttributes(attributes.build()).setAudioFormat(new AudioFormat.Builder().setEncoding(AudioFormat.ENCODING_PCM_16BIT).setSampleRate(48000).setChannelMask(AudioFormat.CHANNEL_OUT_STEREO).build()).setBufferSizeInBytes(samples.length*2).setTransferMode(AudioTrack.MODE_STATIC).build();
                track.write(samples,0,samples.length);track.play();Thread.sleep(650);
            }catch(Exception error){android.util.Log.e("Relay","Test sound failed",error);}finally{if(track!=null){try{track.stop();}catch(Exception ignored){}track.release();}}
        },"relay-test-sound").start();
    }
    @Override public void onDestroy(){stopped=true;sharing=false;connecting=false;stopPlaybackAudio();try{if(socket!=null)socket.close();}catch(Exception ignored){}if(projection!=null){projection.stop();projection=null;}stopForeground(true);super.onDestroy();}
}
