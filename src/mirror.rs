// SPDX-License-Identifier: MPL-2.0
//! The same native window becomes a compact phone mirror for the lifetime of a session.
use super::*;

const BEZEL: Color32 = Color32::from_rgb(18, 21, 27);
const OUTLINE: Color32 = Color32::from_rgb(83, 90, 101);
const BLUE: Color32 = Color32::from_rgb(52, 113, 246);

pub fn window_size(source: [usize; 2], rotation: u8, monitor: Vec2) -> Vec2 {
    let (width, height) = if rotation % 2 == 0 { (source[0], source[1]) } else { (source[1], source[0]) };
    let aspect = width.max(1) as f32 / height.max(1) as f32;
    let limit = (monitor - vec2(100., 120.)).max(vec2(300., 360.));
    if aspect > 1. {
        let screen_width = (limit.x - 32.).min(1200.).min((limit.y - 124.) * aspect);
        vec2(screen_width + 32., screen_width / aspect + 124.)
    } else {
        let screen_height = (limit.y - 56.).min(820.).min((limit.x - 102.) / aspect);
        vec2(screen_height * aspect + 102., screen_height + 56.)
    }
}

pub fn sync_window(app: &mut Relay, ctx: &Context) {
    let geometry = app.last_image.as_ref().map(|frame| (frame.size, app.mirror_rotation));
    if geometry.is_some() && !app.mirror_mode {
        app.menu_size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size())).unwrap_or(vec2(1240., 850.));
        app.menu_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        app.mirror_mode = true;
        eprintln!("Relay: entered phone mirror");
        app.mirror_rotation = 0;
        app.mirror_geometry = None;
        app.help = false;
        app.dialog = false;
        app.fullscreen = false;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(260., 360.)));
        let peer=app.companion.as_ref().map(|c|c.peer_name.lock().unwrap().clone()).unwrap_or_else(||"Phone".into());ctx.send_viewport_cmd(ViewportCommand::Title(format!("Relay · {peer}")));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
    if let Some(frame) = &app.last_image {
        let current = (frame.size, app.mirror_rotation);
        if app.mirror_geometry != Some(current) {
            app.mirror_geometry = Some(current);
            if !app.fullscreen {
                let monitor = ctx.input(|i| i.viewport().monitor_size).unwrap_or(vec2(1920., 1080.));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(window_size(frame.size, app.mirror_rotation, monitor)));
            }
        }
    } else if app.mirror_mode {
        app.mirror_mode = false;
        eprintln!("Relay: returned to main menu");
        app.mirror_geometry = None;
        app.mirror_rotation = 0;
        app.fullscreen = false;
        app.page = 0;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(1080., 800.)));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(app.menu_size));
        ctx.send_viewport_cmd(ViewportCommand::Maximized(app.menu_maximized));
        ctx.send_viewport_cmd(ViewportCommand::Title("Relay — Android, on your desktop".into()));
    }
}

fn header_button(ui: &mut Ui, rect: Rect, close: bool) -> bool {
    let response = ui.interact(rect, Id::new(("phone-header", close)), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 6., if close {Color32::from_rgb(255, 229, 232)} else {Color32::from_gray(235)});
    }
    let stroke = Stroke::new(1.5_f32, Color32::from_rgb(81, 91, 108));
    let center = rect.center();
    if close {
        ui.painter().line_segment([center-vec2(4.,4.),center+vec2(4.,4.)],stroke);
        ui.painter().line_segment([center+vec2(-4.,4.),center+vec2(4.,-4.)],stroke);
    } else {ui.painter().line_segment([center-vec2(4.,0.),center+vec2(4.,0.)],stroke);}
    response.on_hover_text(if close {"Stop mirroring and return to the menu"} else {"Minimize mirror"}).clicked()
}

fn tool(ui: &mut Ui, rect: Rect, index: usize, active: bool) -> bool {
    let response = ui.interact(rect, Id::new(("mirror-tool", index)), Sense::click());
    if response.hovered() || active {ui.painter().rect_filled(rect,8.,Color32::from_rgb(234,241,255));}
    let c = rect.center();
    let stroke = Stroke::new(1.7_f32, if index==4 {Color32::from_rgb(211,70,88)} else {BLUE});
    let p = ui.painter();
    match index {
        0 => for (x,y) in [(-1.,-1.),(1.,-1.),(-1.,1.),(1.,1.)] {
            let corner=c+vec2(x*9.,y*8.);
            p.line_segment([corner,corner-vec2(x*6.,0.)],stroke);
            p.line_segment([corner,corner-vec2(0.,y*6.)],stroke);
        },
        1 => {p.rect_stroke(Rect::from_center_size(c,vec2(23.,17.)),4.,stroke,StrokeKind::Inside);p.circle_stroke(c,4.,stroke);p.line_segment([c+vec2(-6.,-9.),c+vec2(1.,-9.)],stroke);},
        2 => {p.rect_stroke(Rect::from_center_size(c,vec2(13.,22.)),3.,stroke,StrokeKind::Inside);p.line_segment([c+vec2(10.,-5.),c+vec2(10.,7.)],stroke);p.line_segment([c+vec2(10.,7.),c+vec2(15.,2.)],stroke);p.line_segment([c+vec2(10.,7.),c+vec2(5.,2.)],stroke);},
        3 => {p.line_segment([c+vec2(-10.,-4.),c+vec2(-5.,-4.)],stroke);p.line_segment([c+vec2(-5.,-4.),c+vec2(2.,-10.)],stroke);p.line_segment([c+vec2(2.,-10.),c+vec2(2.,10.)],stroke);p.line_segment([c+vec2(2.,10.),c+vec2(-5.,4.)],stroke);p.line_segment([c+vec2(-5.,4.),c+vec2(-10.,4.)],stroke);p.line_segment([c+vec2(-10.,4.),c+vec2(-10.,-4.)],stroke);if active{p.line_segment([c+vec2(7.,-5.),c+vec2(15.,5.)],stroke);p.line_segment([c+vec2(7.,5.),c+vec2(15.,-5.)],stroke);}else{p.line_segment([c+vec2(8.,-5.),c+vec2(12.,0.)],stroke);p.line_segment([c+vec2(12.,0.),c+vec2(8.,5.)],stroke);}},
        _ => {p.rect_stroke(Rect::from_center_size(c,vec2(17.,17.)),4.,stroke,StrokeKind::Inside);},
    }
    response.on_hover_text(["Fullscreen · F11","Save phone screenshot","Rotate the view",if active{"Unmute PC audio"}else{"Mute PC audio"},"Stop mirroring · return to menu"][index]).clicked()
}

fn rotated_uv(x:f32,y:f32,rotation:u8)->Pos2 {
    match rotation%4 {0=>pos2(x,y),1=>pos2(y,1.-x),2=>pos2(1.-x,1.-y),_=>pos2(1.-y,x)}
}
fn paint_screen(ui:&Ui,texture:TextureId,screen:Rect,rotation:u8) {
    let radius=14_f32.min(screen.width()/4.).min(screen.height()/4.);
    let mut mesh=egui::Mesh::with_texture(texture);
    mesh.vertices.push(egui::epaint::Vertex{pos:screen.center(),uv:pos2(0.5,0.5),color:Color32::WHITE});
    for (center,angle) in [(screen.right_top()+vec2(-radius,radius),-90_f32),(screen.right_bottom()-vec2(radius,radius),0.),(screen.left_bottom()+vec2(radius,-radius),90.),(screen.left_top()+vec2(radius,radius),180.)] {
        for step in 0..=8 {let angle=(angle+step as f32*90./8.).to_radians();let pos=center+vec2(angle.cos(),angle.sin())*radius;
            let x=(pos.x-screen.left())/screen.width();let y=(pos.y-screen.top())/screen.height();
            mesh.vertices.push(egui::epaint::Vertex{pos,uv:rotated_uv(x,y,rotation),color:Color32::WHITE});
        }
    }
    let count=mesh.vertices.len()as u32-1;
    for i in 1..=count {mesh.indices.extend_from_slice(&[0,i,if i==count{1}else{i+1}]);}
    ui.painter().add(egui::Shape::mesh(mesh));
}

pub fn render(app: &mut Relay, ctx: &Context) {
    CentralPanel::default().frame(Frame::NONE.fill(Color32::TRANSPARENT)).show(ctx,|ui| {
        let all = ui.max_rect();
        let Some(texture) = &app.texture else {return};
        let source = texture.size_vec2();
        let effective = if app.mirror_rotation%2==0 {source} else {vec2(source.y,source.x)};
        let landscape = effective.x>effective.y;
        let header = Rect::from_min_size(all.min+vec2(8.,0.),vec2(all.width()-16.,30.));
        ui.painter().rect_filled(header,8.,Color32::from_rgb(248,250,254));
        ui.painter().text(header.min+vec2(12.,6.),Align2::LEFT_TOP,"Relay",FontId::proportional(12.),Color32::from_rgb(79,91,111));
        let drag=Rect::from_min_max(header.min,pos2(header.right()-66.,header.bottom()));
        let drag=ui.interact(drag,Id::new("phone-drag-handle"),Sense::click_and_drag());
        if drag.drag_started(){ctx.send_viewport_cmd(ViewportCommand::StartDrag);}
        if drag.double_clicked(){app.fullscreen=!app.fullscreen;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(app.fullscreen));}
        if header_button(ui,Rect::from_min_size(pos2(header.right()-61.,header.top()+2.),vec2(28.,25.)),false){ctx.send_viewport_cmd(ViewportCommand::Minimized(true));}
        let stop=header_button(ui,Rect::from_min_size(pos2(header.right()-31.,header.top()+2.),vec2(28.,25.)),true);
        let (area, toolbar) = if landscape {
            (Rect::from_min_max(all.min+vec2(8.,98.),all.max-vec2(8.,8.)),Rect::from_center_size(pos2(all.center().x,62.),vec2(260.,54.)))
        } else {
            (Rect::from_min_max(all.min+vec2(8.,36.),all.max-vec2(72.,8.)),Rect::from_min_size(pos2(all.right()-62.,70.),vec2(54.,260.)))
        };
        let factor=((area.width()-18.)/effective.x).min((area.height()-18.)/effective.y).max(0.01);
        let screen=Rect::from_center_size(area.center(),effective*factor);
        let bezel=screen.expand(8.);
        let radius=if landscape {24.} else {30.};
        ui.painter().rect_filled(bezel.translate(vec2(0.,2.)).expand(1.),radius,Color32::from_black_alpha(32));
        ui.painter().rect(bezel,radius,BEZEL,Stroke::new(1.5_f32,OUTLINE),StrokeKind::Inside);
        ui.painter().rect_stroke(bezel.shrink(3.),radius-3.,Stroke::new(0.7_f32,Color32::from_gray(55)),StrokeKind::Inside);
        paint_screen(ui,texture.id(),screen,app.mirror_rotation);
        ui.painter().rect(toolbar,11.,Color32::from_rgb(251,252,255),Stroke::new(1.0_f32,Color32::from_rgb(222,228,239)),StrokeKind::Inside);
        let mut clicked=None;
        for i in 0..5 {
            let offset=if landscape {vec2(5.+i as f32*50.,5.)} else {vec2(5.,5.+i as f32*50.)};
            if tool(ui,Rect::from_min_size(toolbar.min+offset,vec2(44.,44.)),i,(i==0&&app.fullscreen)||(i==3&&app.companion.as_ref().is_some_and(|c|c.audio.muted.load(std::sync::atomic::Ordering::Relaxed)))){clicked=Some(i);}
        }
        if stop||clicked==Some(4){app.end_native_session(ctx);return;}
        match clicked {
            Some(0)=>{app.fullscreen=!app.fullscreen;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(app.fullscreen));},
            Some(1)=>app.capture_native(),
            Some(3)=>{if let Some(c)=&app.companion{c.audio.toggle();}},
            Some(2)=>{app.mirror_rotation=(app.mirror_rotation+1)%4;sync_window(app,ctx);},
            _=>{}
        }
    });
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn mirror_sizes_follow_orientation_and_fit_display() {
        let display=vec2(1920.,1080.);
        let portrait=window_size([590,1280],0,display);
        let landscape=window_size([1280,590],0,display);
        assert!(portrait.y>portrait.x);
        assert!(landscape.x>landscape.y);
        assert!(portrait.y<=display.y-100.);
        assert!(landscape.x<=display.x-100.);
        assert_eq!(landscape,window_size([590,1280],1,display));
    }
    #[test] fn clockwise_rotation_maps_texture_corners() {
        assert_eq!(rotated_uv(0.,0.,0),pos2(0.,0.));
        assert_eq!(rotated_uv(0.,0.,1),pos2(0.,1.));
        assert_eq!(rotated_uv(1.,0.,1),pos2(0.,0.));
        assert_eq!(rotated_uv(0.,0.,4),rotated_uv(0.,0.,0));
    }
    #[test] fn small_display_never_gets_a_giant_mirror() {
        for source in [[590,1280],[1280,590],[720,1280]] {
            let size=window_size(source,0,vec2(1024.,768.));
            assert!(size.x<=924.&&size.y<=648.);
        }
    }
}
