// SPDX-License-Identifier: MPL-2.0
//! Small native egui/winit runner with WGPU rendering and screenshot readback.
use std::{sync::Arc,collections::HashMap,time::{Instant,Duration}};
use winit::{application::ApplicationHandler,event::WindowEvent,event_loop::{ActiveEventLoop,EventLoop,ControlFlow},window::{Window,WindowId}};
pub type Result<T=()> = std::result::Result<T,Box<dyn std::error::Error>>;
pub trait Storage {fn get_string(&self,key:&str)->Option<String>;fn set_string(&mut self,key:&str,value:String);}
#[derive(Default)] struct DiskStorage(HashMap<String,String>);
impl Storage for DiskStorage {fn get_string(&self,key:&str)->Option<String>{self.0.get(key).cloned()}fn set_string(&mut self,key:&str,value:String){self.0.insert(key.into(),value);}}
pub struct CreationContext<'a>{pub egui_ctx:egui::Context,pub storage:Option<&'a dyn Storage>}
pub struct Frame;
pub trait App {fn update(&mut self,ctx:&egui::Context,frame:&mut Frame);fn save(&mut self,_s:&mut dyn Storage){}fn on_exit(&mut self,_gl:Option<&()>){}fn close_requested(&mut self,_ctx:&egui::Context)->bool{true}}
#[derive(Default)]pub struct NativeOptions {
 pub viewport:egui::ViewportBuilder,
 pub background_render:bool,
 pub event_loop_builder:Option<Box<dyn FnOnce(&mut winit::event_loop::EventLoopBuilder<()>)>>,
}
type Factory = Box<dyn FnOnce(&CreationContext<'_>)->Result<Box<dyn App>>>;
struct Graphics {state:egui_winit::State,renderer:egui_wgpu::Renderer,surface:wgpu::Surface<'static>,queue:wgpu::Queue,device:wgpu::Device,config:wgpu::SurfaceConfiguration,window:Arc<Window>}
struct Runner{options:NativeOptions,factory:Option<Factory>,app:Option<Box<dyn App>>,graphics:Option<Graphics>,ctx:egui::Context,storage:DiskStorage,rx:egui_wgpu::capture::CaptureReceiver,tx:egui_wgpu::capture::CaptureSender,next:Instant,closed:bool}
impl Runner {
 fn close(&mut self,event_loop:&ActiveEventLoop){if !self.closed{if let Some(a)=&mut self.app{a.save(&mut self.storage);a.on_exit(None);}if let Ok(s)=serde_json::to_string(&self.storage.0){let _=crate::paths::save_settings(s.as_bytes());}self.closed=true;self.ctx.set_request_repaint_callback(|_|{});self.graphics.take();}event_loop.exit();}
 fn draw(&mut self,event_loop:&ActiveEventLoop){self.next=Instant::now()+Duration::from_millis(16);let Some(g)=&mut self.graphics else{return};let size=g.window.inner_size();if size.width==0||size.height==0{return}if g.config.width!=size.width||g.config.height!=size.height{g.config.width=size.width;g.config.height=size.height;g.surface.configure(&g.device,&g.config);}
 // Acquire before consuming screenshot commands, so a transient surface error cannot lose one.
 let frame=match g.surface.get_current_texture(){Ok(f)=>f,Err(_)=>{g.surface.configure(&g.device,&g.config);return}};
 let mut input=g.state.take_egui_input(&g.window);while let Ok((id,data,image))=self.rx.try_recv(){for user_data in data{input.events.push(egui::Event::Screenshot{viewport_id:id,user_data,image:Arc::new(image.clone())});}}
 let output=self.ctx.run(input,|ctx|{self.app.as_mut().unwrap().update(ctx,&mut Frame)});g.state.handle_platform_output(&g.window,output.platform_output);
 let mut screenshot=vec![];let mut close=false;let mut delay=Duration::from_millis(200);for v in output.viewport_output.values(){delay=delay.min(v.repaint_delay);for cmd in &v.commands{match cmd{egui::ViewportCommand::Close=>close=true,egui::ViewportCommand::Screenshot(data)=>screenshot.push(data.clone()),egui::ViewportCommand::InnerSize(size)=>{let _=g.window.request_inner_size(winit::dpi::LogicalSize::new(size.x,size.y));},
 egui::ViewportCommand::MinInnerSize(size)=>g.window.set_min_inner_size(Some(winit::dpi::LogicalSize::new(size.x,size.y))),
 egui::ViewportCommand::Decorations(value)=>g.window.set_decorations(*value),
 egui::ViewportCommand::Title(title)=>g.window.set_title(title),
 egui::ViewportCommand::StartDrag=>{let _=g.window.drag_window();},
 egui::ViewportCommand::Minimized(value)=>g.window.set_minimized(*value),
 egui::ViewportCommand::Maximized(value)=>g.window.set_maximized(*value),
 egui::ViewportCommand::Focus=>g.window.focus_window(),
 egui::ViewportCommand::Fullscreen(enabled)=>g.window.set_fullscreen(if *enabled{Some(winit::window::Fullscreen::Borderless(None))}else{None}),_=>{}}}}
 if close{drop(frame);self.close(event_loop);return}
 let mut capture=if screenshot.is_empty(){None}else{Some(egui_wgpu::capture::CaptureState::new(&g.device,&frame.texture))};
 let view=if let Some(c)=&capture{c.texture.create_view(&Default::default())}else{frame.texture.create_view(&Default::default())};
 let jobs=self.ctx.tessellate(output.shapes,output.pixels_per_point);let screen=egui_wgpu::ScreenDescriptor{size_in_pixels:[size.width,size.height],pixels_per_point:output.pixels_per_point};for(id,delta)in &output.textures_delta.set{g.renderer.update_texture(&g.device,&g.queue,*id,delta);}
 let mut encoder=g.device.create_command_encoder(&Default::default());let buffers=g.renderer.update_buffers(&g.device,&g.queue,&mut encoder,&jobs,&screen);
 {let pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:Some("Relay"),color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&view,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),store:wgpu::StoreOp::Store},depth_slice:None})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None});g.renderer.render(&mut pass.forget_lifetime(),&jobs,&screen);}
 let buffer=capture.as_mut().map(|c|c.copy_textures(&g.device,&frame,&mut encoder));g.queue.submit(buffers.into_iter().chain(std::iter::once(encoder.finish())));if let(Some(c),Some(b))=(capture,buffer){c.read_screen_rgba(self.ctx.clone(),b,screenshot,self.tx.clone(),egui::ViewportId::ROOT);}g.window.pre_present_notify();frame.present();
 // Poll after presenting; never wait indefinitely for a covered window's presentation.
 let _=g.device.poll(wgpu::PollType::Poll);
 for id in &output.textures_delta.free{g.renderer.free_texture(id);}
 self.next=Instant::now()+delay.max(Duration::from_millis(16));event_loop.set_control_flow(ControlFlow::WaitUntil(self.next));
 }
}
impl ApplicationHandler for Runner {
 fn user_event(&mut self,_event_loop:&ActiveEventLoop,_event:()){self.next=Instant::now();if let Some(g)=&self.graphics{g.window.request_redraw();}}
 fn resumed(&mut self,event_loop:&ActiveEventLoop){if self.graphics.is_some(){return}let v=&self.options.viewport;let s=v.inner_size.unwrap_or(egui::vec2(1280.,850.));let min=v.min_inner_size.unwrap_or(egui::vec2(1100.,780.));let icon=image::load_from_memory(include_bytes!("../assets/relay-icon.png")).ok().and_then(|image|{let rgba=image.to_rgba8();winit::window::Icon::from_rgba(rgba.to_vec(),rgba.width(),rgba.height()).ok()});let attrs=Window::default_attributes().with_window_icon(icon).with_title(v.title.clone().unwrap_or("Relay".into())).with_inner_size(winit::dpi::LogicalSize::new(s.x,s.y)).with_min_inner_size(winit::dpi::LogicalSize::new(min.x,min.y)).with_active(v.active.unwrap_or(true)).with_transparent(true);#[cfg(target_os="windows")]
 let attrs=attrs.with_window_level(match v.window_level.unwrap_or(egui::WindowLevel::Normal){egui::WindowLevel::AlwaysOnBottom=>winit::window::WindowLevel::AlwaysOnBottom,egui::WindowLevel::AlwaysOnTop=>winit::window::WindowLevel::AlwaysOnTop,egui::WindowLevel::Normal=>winit::window::WindowLevel::Normal});
 #[cfg(target_os="linux")] let attrs={use winit::platform::wayland::WindowAttributesExtWayland;attrs.with_name("relay","Relay")};
 let window=Arc::new(event_loop.create_window(attrs).expect("Create desktop window"));
 let instance=wgpu::Instance::new(&wgpu::InstanceDescriptor {backends: if cfg!(target_os="linux"){wgpu::Backends::VULKAN}else{wgpu::Backends::all()},..Default::default()});let surface=instance.create_surface(window.clone()).expect("Create graphics surface");let adapter=pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{compatible_surface:Some(&surface),power_preference:wgpu::PowerPreference::LowPower,..Default::default()})).expect("Find graphics adapter");let(device,queue)=pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor{label:Some("Relay"),..Default::default()})).expect("Create graphics device");let size=window.inner_size();let mut config=surface.get_default_config(&adapter,size.width,size.height).unwrap();let caps=surface.get_capabilities(&adapter);config.format=caps.formats.iter().copied().find(|f|matches!(f,wgpu::TextureFormat::Bgra8Unorm|wgpu::TextureFormat::Rgba8Unorm)).unwrap_or(config.format);config.alpha_mode=caps.alpha_modes.iter().copied().find(|m|matches!(m,wgpu::CompositeAlphaMode::PreMultiplied)).or_else(||caps.alpha_modes.iter().copied().find(|m|matches!(m,wgpu::CompositeAlphaMode::PostMultiplied))).unwrap_or(config.alpha_mode);config.desired_maximum_frame_latency=1;if caps.present_modes.contains(&wgpu::PresentMode::Mailbox){config.present_mode=wgpu::PresentMode::Mailbox;}surface.configure(&device,&config);
 let state=egui_winit::State::new(self.ctx.clone(),egui::ViewportId::ROOT,&window,Some(window.scale_factor() as f32),None,None);let renderer=egui_wgpu::Renderer::new(&device,config.format,egui_wgpu::RendererOptions::default());self.graphics=Some(Graphics{window,surface,device,queue,config,renderer,state});let cc=CreationContext{egui_ctx:self.ctx.clone(),storage:Some(&self.storage)};self.app=Some(self.factory.take().unwrap()(&cc).expect("Create Relay"));self.graphics.as_ref().unwrap().window.request_redraw();
 }
 fn window_event(&mut self,event_loop:&ActiveEventLoop,_id:WindowId,event:WindowEvent){if matches!(event,WindowEvent::CloseRequested){let exit=self.app.as_mut().is_none_or(|app|app.close_requested(&self.ctx));if exit{self.close(event_loop);}else if let Some(g)=&self.graphics{g.window.request_redraw();self.next=Instant::now();}return}if let Some(g)=&mut self.graphics{let response=g.state.on_window_event(&g.window,&event);if response.repaint{g.window.request_redraw();}}if matches!(event,WindowEvent::RedrawRequested)&&!self.options.background_render{self.draw(event_loop)}}
 fn about_to_wait(&mut self,event_loop:&ActiveEventLoop){
 if Instant::now()>=self.next {
  // Wayland can withhold RedrawRequested for inactive/covered windows. Screenshot
  // checks must advance independently of compositor redraw and focus events.
  if self.options.background_render {self.draw(event_loop);}
  else if let Some(g)=&self.graphics{g.window.request_redraw();}
 }
 event_loop.set_control_flow(ControlFlow::WaitUntil(self.next));
 }
}
pub fn run_native(_name:&str,mut options:NativeOptions,factory:Factory)->Result {let mut builder=EventLoop::builder();if let Some(configure)=options.event_loop_builder.take(){configure(&mut builder);}let event_loop=builder.build()?;let storage=std::fs::read_to_string(crate::paths::settings_file()).ok().and_then(|s|serde_json::from_str(&s).ok()).map(DiskStorage).unwrap_or_default();let(tx,rx)=egui_wgpu::capture::capture_channel();let mut runner=Runner{options,factory:Some(factory),app:None,graphics:None,ctx:egui::Context::default(),storage,rx,tx,next:Instant::now(),closed:false};let proxy=event_loop.create_proxy();runner.ctx.set_request_repaint_callback(move|info|{if info.delay.is_zero(){let _=proxy.send_event(());}});event_loop.run_app(&mut runner)?;Ok(())}

/// Deterministic GPU preview when a CI environment has no desktop connection.
/// Normal --screenshot always uses the real window above.
pub fn render_headless(path:std::path::PathBuf,factory:Factory)->Result {
 let ctx=egui::Context::default();let mut app=factory(&CreationContext{egui_ctx:ctx.clone(),storage:None})?;
 let instance=wgpu::Instance::new(&wgpu::InstanceDescriptor {backends: if cfg!(target_os="linux"){wgpu::Backends::VULKAN}else{wgpu::Backends::all()},..Default::default()});let adapter=pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{power_preference:wgpu::PowerPreference::LowPower,..Default::default()}))?;let(device,queue)=pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
 let texture=device.create_texture(&wgpu::TextureDescriptor{label:Some("Relay preview"),size:wgpu::Extent3d{width:1280,height:850,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});let view=texture.create_view(&Default::default());let mut renderer=egui_wgpu::Renderer::new(&device,wgpu::TextureFormat::Rgba8Unorm,Default::default());
 for n in 0..120 {let input=egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1280.,850.))),time:Some(n as f64/60.),..Default::default()};let output=ctx.run(input,|ctx|app.update(ctx,&mut Frame));let jobs=ctx.tessellate(output.shapes,1.);for(id,delta)in &output.textures_delta.set{renderer.update_texture(&device,&queue,*id,delta);}let screen=egui_wgpu::ScreenDescriptor{size_in_pixels:[1280,850],pixels_per_point:1.};let mut encoder=device.create_command_encoder(&Default::default());let buffers=renderer.update_buffers(&device,&queue,&mut encoder,&jobs,&screen);{let pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:None,color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&view,resolve_target:None,depth_slice:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),store:wgpu::StoreOp::Store}})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None});renderer.render(&mut pass.forget_lifetime(),&jobs,&screen);}queue.submit(buffers.into_iter().chain(std::iter::once(encoder.finish())));for id in &output.textures_delta.free{renderer.free_texture(id);}}
 let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:1280*850*4,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});let mut encoder=device.create_command_encoder(&Default::default());encoder.copy_texture_to_buffer(texture.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(1280*4),rows_per_image:None}},texture.size());queue.submit([encoder.finish()]);let(tx,rx)=std::sync::mpsc::channel();buffer.slice(..).map_async(wgpu::MapMode::Read,move|r|{let _=tx.send(r);});device.poll(wgpu::PollType::wait_indefinitely())?;rx.recv()??;if let Some(p)=path.parent(){std::fs::create_dir_all(p)?;}image::save_buffer(path,&buffer.slice(..).get_mapped_range(),1280,850,image::ColorType::Rgba8)?;buffer.unmap();app.on_exit(None);Ok(())
}
