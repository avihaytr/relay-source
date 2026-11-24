// SPDX-License-Identifier: MPL-2.0
use super::*;

const BLUE: Color32 = Color32::from_rgb(52, 113, 246);
const INK: Color32 = Color32::from_rgb(30, 43, 68);
const SUB: Color32 = Color32::from_rgb(119, 131, 154);
const BORDER: Color32 = Color32::from_rgb(229, 234, 243);
const SOFT: Color32 = Color32::from_rgb(241, 246, 255);

fn label(ui: &Ui, p: Pos2, value: impl Into<String>, size: f32, color: Color32) {
    ui.painter().text(p, Align2::LEFT_TOP, value.into(), FontId::proportional(size), color);
}
fn center(ui: &Ui, p: Pos2, value: &str, size: f32, color: Color32) {
    ui.painter().text(p, Align2::CENTER_TOP, value, FontId::proportional(size), color);
}
fn surface(ui: &Ui, r: Rect, color: Color32) {
    ui.painter().rect(r, 16., color, Stroke::new(1.0_f32, BORDER), StrokeKind::Inside);
}
fn action(ui: &mut Ui, r: Rect, value: &str, primary: bool) -> bool {
    ui.put(r, Button::new(RichText::new(value).size(14.).color(if primary {Color32::WHITE} else {INK}))
        .fill(if primary {BLUE} else {Color32::WHITE})
        .stroke(Stroke::new(1.0_f32, if primary {BLUE} else {BORDER}))
        .corner_radius(8.)).clicked()
}
fn tabs(ui: &mut Ui, r: Rect, options: &[&str], selected: &mut usize) {
    ui.painter().rect_filled(r, 9., SOFT);
    let width = (r.width()-8.)/options.len() as f32;
    for (i, name) in options.iter().enumerate() {
        let item=Rect::from_min_size(r.min+vec2(4.+i as f32*width,4.),vec2(width,r.height()-8.));
        let active=*selected==i;
        if ui.put(item,Button::new(RichText::new(*name).size(13.).color(if active {BLUE} else {SUB}))
            .fill(if active {Color32::WHITE} else {Color32::TRANSPARENT})
            .stroke(Stroke::NONE).corner_radius(6.)).clicked(){*selected=i;}
    }
}
pub fn qr(ui: &mut Ui, value: &str, r: Rect) {
    let id=Id::new(("connection-qr",value));
    let cached=ui.ctx().data_mut(|d|d.get_temp::<std::sync::Arc<qrcode::QrCode>>(id));
    let code=match cached {
        Some(code)=>code,
        None=>{let Ok(code)=qrcode::QrCode::new(value.as_bytes()) else{return};let code=std::sync::Arc::new(code);ui.ctx().data_mut(|d|d.insert_temp(id,code.clone()));code}
    };
    ui.painter().rect_filled(r,10.,Color32::WHITE);
    let modules=code.width();let cell=(r.width()/(modules+8) as f32).floor().max(1.);
    let start=r.center()-vec2(modules as f32*cell,modules as f32*cell)/2.;
    for y in 0..modules {for x in 0..modules {if code[(x,y)]==qrcode::Color::Dark {
        ui.painter().rect_filled(Rect::from_min_size(start+vec2(x as f32*cell,y as f32*cell),vec2(cell,cell)),0.,Color32::from_rgb(28,40,60));
    }}}
}

/// Original Relay illustration; drawn as vectors, separate from actual session state.
fn illustration(ui: &Ui, area: Rect) {
    let p=ui.painter();let c=area.center();let scale=(area.width()/570.).min(area.height()/355.).min(1.);
    let at=|x:f32,y:f32|c+vec2(x*scale,y*scale);
    p.circle_filled(at(0.,-10.),150.*scale,Color32::from_rgb(234,241,255));
    p.circle_filled(at(140.,-94.),18.*scale,Color32::from_rgb(220,234,255));
    p.circle_filled(at(-175.,67.),10.*scale,Color32::from_rgb(220,234,255));
    let monitor=Rect::from_min_max(at(-177.,-100.),at(143.,94.));
    p.rect_filled(monitor.translate(vec2(0.,8.*scale)),14.*scale,Color32::from_rgb(219,230,250));
    p.rect(monitor,14.*scale,Color32::WHITE,Stroke::new(2.*scale,Color32::from_rgb(186,207,244)),StrokeKind::Inside);
    let screen=monitor.shrink(9.*scale);p.rect_filled(screen,8.*scale,BLUE);
    p.with_clip_rect(screen.shrink(1.*scale)).circle_filled(at(53.,-80.),66.*scale,Color32::from_rgb(83,142,255));
    p.with_clip_rect(screen.shrink(1.*scale)).circle_filled(at(-57.,55.),87.*scale,Color32::from_rgb(43,100,225));
    p.rect_filled(Rect::from_min_max(at(-36.,94.),at(2.,124.)),3.*scale,Color32::from_rgb(186,207,244));
    p.rect_filled(Rect::from_min_max(at(-72.,120.),at(39.,129.)),5.*scale,Color32::from_rgb(186,207,244));
    let phone=Rect::from_min_max(at(85.,-47.),at(180.,133.));
    p.rect_filled(phone.translate(vec2(-6.*scale,5.*scale)),17.*scale,Color32::from_rgb(207,221,247));
    p.rect(phone,17.*scale,Color32::WHITE,Stroke::new(2.*scale,Color32::from_rgb(171,195,234)),StrokeKind::Inside);
    p.rect_filled(phone.shrink(7.*scale),11.*scale,Color32::from_rgb(217,232,255));
    p.rect_filled(Rect::from_min_size(at(110.,-36.),vec2(42.*scale,5.*scale)),3.*scale,Color32::WHITE);
    p.circle_filled(at(132.,34.),24.*scale,BLUE);
    p.line_segment([at(122.,34.),at(142.,34.)],Stroke::new(3.*scale,Color32::WHITE));
    p.line_segment([at(135.,27.),at(142.,34.)],Stroke::new(3.*scale,Color32::WHITE));
    p.line_segment([at(135.,41.),at(142.,34.)],Stroke::new(3.*scale,Color32::WHITE));
    for i in 0..3 {p.circle_filled(at(40.+i as f32*14.,25.),2.5*scale,Color32::WHITE);}
}

pub fn render(app: &mut Relay, ctx: &Context) {
    if app.easy && app.companion.is_none() {
        match companion::Companion::start(ctx.clone(),app.settings.device_name.clone(),app.settings.wireless_profile) {Ok(c)=>app.companion=Some(c),Err(e)=>app.status=format!("Could not start receiver: {e}")}
        app.easy=false;
    }
    if app.companion.as_ref().is_some_and(|c|c.desktop.pending.load(std::sync::atomic::Ordering::Relaxed)||c.desktop.active.load(std::sync::atomic::Ordering::Relaxed)){desktop_session(app,ctx);return;}
    CentralPanel::default().frame(Frame::NONE.fill(Color32::from_rgb(246,248,252))).show(ctx,|ui|{
        let all=ui.max_rect();let w=all.width();let h=all.height();
        ui.painter().rect_filled(Rect::from_min_size(all.min,vec2(w,78.)),0.,Color32::WHITE);
        ui.painter().hline(0.0..=w,78.,Stroke::new(1.0_f32,BORDER));
        let brand=Rect::from_min_size(pos2(24.,14.),vec2(50.,50.));
        // Draw at the actual viewport scale; no minified bitmap or texture sampling.
        let painter=ui.painter();let origin=brand.min;
        let at=|x:f32,y:f32|origin+vec2(x,y);
        painter.rect_filled(brand,11.,Color32::from_rgb(7,83,245));
        painter.rect_stroke(Rect::from_min_max(at(8.,11.),at(38.,34.)),5.,Stroke::new(2.2_f32,Color32::WHITE),StrokeKind::Middle);
        painter.line_segment([at(21.,34.),at(21.,38.)],Stroke::new(2.4_f32,Color32::WHITE));
        painter.line_segment([at(18.,38.),at(26.,38.)],Stroke::new(2.4_f32,Color32::WHITE));
        painter.rect(Rect::from_min_max(at(28.,18.),at(43.,40.)),4.,Color32::from_rgb(7,83,245),Stroke::new(2.0_f32,Color32::WHITE),StrokeKind::Middle);
        painter.line_segment([at(32.,29.),at(39.,29.)],Stroke::new(2.0_f32,Color32::WHITE));
        painter.add(egui::Shape::line(vec![at(36.,26.),at(39.,29.),at(36.,32.)],Stroke::new(2.0_f32,Color32::WHITE)));
        label(ui,pos2(86.,20.),"Relay",25.,INK);
        label(ui,pos2(160.,30.),"Screen mirroring",12.,SUB);
        for (i,title) in ["Mirror","Captures","Settings"].iter().enumerate(){
            let r=Rect::from_min_size(pos2(w/2.-155.+i as f32*111.,18.),vec2(102.,42.));
            if ui.put(r,Button::new(RichText::new(*title).size(14.).color(if app.page==i {BLUE}else{SUB}))
                .fill(if app.page==i {SOFT}else{Color32::TRANSPARENT}).stroke(Stroke::NONE).corner_radius(8.)).clicked(){app.page=i;if i==1{app.files();}}
        }
        if action(ui,Rect::from_min_size(pos2(w-118.,23.),vec2(88.,32.)),"Help",false){app.help=true;}
        let body=Rect::from_min_max(pos2(28.,102.),pos2(w-28.,h-62.));
        match app.page {0=>mirror(app,ctx,ui,body),1=>captures(app,ui,body),_=>settings(app,ui,body)}
        ui.painter().hline(0.0..=w,h-42.,Stroke::new(1.0_f32,BORDER));
        let wireless=app.tab==2;
        let status=if app.page==0 && !wireless {app.status.clone()} else {app.companion.as_ref().map(|c|c.status.lock().unwrap().clone()).unwrap_or_else(||app.status.clone())};
        let live=app.texture.is_some()||app.child.is_some();
        ui.painter().circle_filled(pos2(34.,h-21.),3.,if live{Color32::from_rgb(36,173,128)}else{BLUE});
        label(ui,pos2(46.,h-29.),status.chars().take(128).collect::<String>(),11.,SUB);
        label(ui,pos2(w-148.,h-29.),if wireless {"Local Wi-Fi connection"} else {"USB / advanced"},10.,SUB);
    });
}

fn mirror(app: &mut Relay,ctx: &Context,ui: &mut Ui,body: Rect) {

    let left_width=(body.width()*0.6).min(body.width()-350.);
    let left=Rect::from_min_max(body.min,pos2(body.left()+left_width-20.,body.bottom()));
    let right=Rect::from_min_max(pos2(body.left()+left_width,body.top()),body.max);
    surface(ui,left,Color32::WHITE);surface(ui,right,Color32::WHITE);
    label(ui,left.min+vec2(28.,25.),"Phone screen mirroring",24.,INK);
    label(ui,left.min+vec2(28.,62.),"More space for everything on your phone.",13.,SUB);
    let mut mode=if app.tab==2{0}else{1};
    tabs(ui,Rect::from_min_size(left.min+vec2(28.,98.),vec2(left.width()-56.,44.)),&["Wireless connection","USB / advanced"],&mut mode);
    if mode==0{app.tab=2;}else if app.tab==2{app.tab=0;}
    if mode==0 {
        let illustration_area=Rect::from_min_max(left.min+vec2(25.,155.),pos2(left.right()-25.,left.bottom()-158.));
        illustration(ui,illustration_area);
        let y=left.bottom()-151.;
        center(ui,pos2(left.center().x,y),"Your computer is ready",21.,INK);
        center(ui,pos2(left.center().x,y+35.),"Open Relay on your phone and select this computer.",13.,SUB);
        let name=app.companion.as_ref().map(|c|c.computer_name.clone()).unwrap_or_else(||"Computer".into());
        let ready=app.companion.is_some();
        if app.companion.as_ref().is_some_and(|c|!c.discovery){center(ui,pos2(left.center().x,y+61.),"Nearby discovery unavailable. Use the QR code.",11.,SUB);}
        let r=Rect::from_center_size(pos2(left.center().x,y+91.),vec2(250.,44.));
        if action(ui,r,&if ready{format!("Relay · {name}")}else{"Enable wireless connection".into()},!ready){app.easy=true;}
    } else {
        usb_setup(app, ctx, ui, left, right);
        return;
    }
    label(ui,right.min+vec2(26.,27.),"Connect your phone",20.,INK);
    label(ui,right.min+vec2(26.,59.),"Keep both devices on the same Wi-Fi.",12.,SUB);
    let mut qr_mode=usize::from(app.qr_install);
    tabs(ui,Rect::from_min_size(right.min+vec2(24.,94.),vec2(right.width()-48.,40.)),&["Scan QR code","Get phone app"],&mut qr_mode);app.qr_install=qr_mode==1;
    if let Some(c)=&app.companion {
        let size=(right.width()-92.).min(262.);
        let qr_rect=Rect::from_center_size(pos2(right.center().x,right.top()+157.+size/2.),vec2(size,size));
        ui.painter().rect_stroke(qr_rect.expand(1.),12.,Stroke::new(1.0_f32,BORDER),StrokeKind::Outside);
        let y=qr_rect.bottom()+18.;
        if app.qr_install {
            center(ui,qr_rect.center()-vec2(0.,35.),"Install from a verified build",16.,INK);
            center(ui,qr_rect.center(),"Build the APK locally; transfer with USB.",12.,SUB);
            center(ui,qr_rect.center()+vec2(0.,28.),"No APK is served over the network.",12.,SUB);
        } else if let Ok(link)=c.pairing_link(){qr(ui,&link,qr_rect);}
        center(ui,pos2(right.center().x,y),"Scan QR or enter PIN on your phone",13.,INK);
        if let Ok(pin)=c.pairing_pin(){center(ui,pos2(right.center().x,y+24.),&format!("Pairing PIN   {pin}"),18.,BLUE);center(ui,pos2(right.center().x,y+48.),"Expires in 2 minutes · locks after five incorrect tries",10.,SUB);}
        if action(ui,Rect::from_min_size(pos2(right.left()+24.,y+70.),vec2(right.width()-48.,37.)),"New pairing QR / PIN",false){let _=c.pairing.lock().unwrap().refresh();}
    } else {
        center(ui,pos2(right.center().x,right.center().y-30.),"Wireless connection is off",15.,INK);
        if action(ui,Rect::from_center_size(right.center()+vec2(0.,30.),vec2(200.,42.)),"Enable connection",true){app.easy=true;}
    }
    connection_steps(ui, right, [
        "Open Relay on both devices",
        "Pair by QR or PIN; compare and approve",
        "Choose a direction; allow screen capture",
    ]);
}

fn connection_steps(ui: &Ui, panel: Rect, steps: [&str; 3]) {
    let y = panel.bottom() - 104.;
    ui.painter().hline(panel.left()+25.0..=panel.right()-25.,y-18.,Stroke::new(1.0_f32,BORDER));
    for (i,line) in steps.into_iter().enumerate() {
        let p=pos2(panel.left()+27.,y+i as f32*27.);
        ui.painter().circle_filled(p+vec2(8.,8.),9.,SOFT);
        center(ui,p+vec2(8.,0.),&(i+1).to_string(),10.,BLUE);
        label(ui,p+vec2(26.,0.),line,11.,SUB);
    }
}

fn usb_setup(app: &mut Relay, ctx: &Context, ui: &mut Ui, left: Rect, right: Rect) {
    label(ui,left.min+vec2(28.,168.),"Android devices",20.,INK);
    if action(ui, Rect::from_min_size(pos2(left.right()-146.,left.top()+163.),vec2(118.,36.)),
        if app.scanning {"Scanning…"} else {"Refresh"}, false) {
        app.refresh();
    }
    let list = Rect::from_min_max(left.min+vec2(28.,216.), left.max-vec2(28.,118.));
    let mut rows=ui.new_child(UiBuilder::new().max_rect(list));
    rows.set_clip_rect(list);
    if app.devices.is_empty() {
        center(&rows,pos2(list.center().x,list.center().y-22.),
            if app.scanning {"Looking for Android devices"} else {"No Android devices found"},19.,INK);
        center(&rows,pos2(list.center().x,list.center().y+11.),
            "Connect a data cable and allow USB debugging on Android.",12.,SUB);
    } else {
        ScrollArea::vertical().id_salt("usb-devices").max_height(list.height()).show(&mut rows, |ui| {
            for device in &app.devices {
                let selected=app.selected.as_ref()==Some(&device.serial);
                let name=format!("{}  ·  {}  ·  {}",device.name,
                    if device.wifi {"Wi-Fi"} else {"USB"},
                    if device.state=="device" {"Ready"} else {&device.state});
                if ui.add_sized([ui.available_width(),42.],Button::new(RichText::new(name)
                    .color(if selected {BLUE} else {INK}))
                    .fill(if selected {SOFT} else {Color32::WHITE})
                    .stroke(Stroke::new(1.0_f32,BORDER)).corner_radius(8.)).clicked() {
                    app.selected=Some(device.serial.clone());
                }
            }
        });
    }
    let controls=left.bottom()-94.;
    let ready=app.chosen().is_some();
    let active=app.child.is_some();
    let primary=Rect::from_center_size(pos2(left.center().x,controls+22.),vec2(218.,44.));
    let title=if active {"Stop mirroring"} else {"Start mirroring"};
    if ui.put(primary,Button::new(RichText::new(title).size(14.).color(Color32::WHITE))
        .fill(if active || ready {BLUE} else {SUB}).sense(if active || ready {Sense::click()} else {Sense::hover()})
        .corner_radius(8.)).clicked() {
        if active {app.stop();} else {app.start(ctx,false);}
    }
    if active && action(ui,Rect::from_center_size(pos2(left.center().x,controls+67.),vec2(218.,32.)),
        "Take screenshot",false) {
        app.capture();
    }
    label(ui,right.min+vec2(26.,27.),"Connect with scrcpy",20.,INK);
    label(ui,right.min+vec2(26.,59.),"Mirror and control your Android device.",12.,SUB);
    illustration(ui,Rect::from_min_max(right.min+vec2(20.,100.),right.max-vec2(20.,166.)));
    if action(ui,Rect::from_center_size(pos2(right.center().x,right.bottom()-145.),vec2(244.,40.)),
        "Advanced Wi-Fi pairing",false) {
        app.dialog=true;
    }
    connection_steps(ui,right,[
        "Enable USB debugging in Developer options",
        "Connect with a data cable and allow debugging",
        "Select your device and start mirroring",
    ]);
}


fn captures(app:&mut Relay,ui:&mut Ui,r:Rect){
    surface(ui,r,Color32::WHITE);let mut child=ui.new_child(UiBuilder::new().max_rect(r.shrink(28.)));
    child.horizontal(|ui|{ui.heading("My captures");if ui.button("Refresh").clicked(){app.files();}if ui.button("Open folder").clicked(){let _=std::fs::create_dir_all(&app.settings.folder);let _=Command::new("xdg-open").arg(&app.settings.folder).spawn();}});
    child.add_space(12.);child.label("Screenshots and recordings saved on this computer.");child.add_space(25.);
    if app.capture_files.is_empty(){child.add_space(80.);child.vertical_centered(|ui|{ui.heading("Your captures will appear here");ui.label("Use Screenshot beside a live mirror, or record through the advanced USB mode.");});}
    ScrollArea::vertical().show(&mut child,|ui|{for path in &app.capture_files{ui.horizontal(|ui|{ui.label(path.file_name().unwrap_or_default().to_string_lossy());if ui.button("Open").clicked(){let _=Command::new("xdg-open").arg(path).spawn();}});ui.add_space(12.);}});
}

fn settings(app:&mut Relay,ui:&mut Ui,r:Rect){
    surface(ui,r,Color32::WHITE);let mut child=ui.new_child(UiBuilder::new().max_rect(r.shrink(28.)));
    ScrollArea::vertical().show(&mut child,|ui|{
        ui.heading("Settings");ui.add_space(18.);ui.heading("This computer");ui.label("Device name shown on your phone");ui.add(TextEdit::singleline(&mut app.settings.device_name).desired_width(420.));
        ui.add_space(15.);ui.label("Wi-Fi quality limit");ComboBox::from_id_salt("wireless-profile").selected_text(profile::get(app.settings.wireless_profile).label).show_ui(ui,|ui|{for id in 0..3{ui.selectable_value(&mut app.settings.wireless_profile,id,profile::get(id).label);}});
        let quality=profile::get(app.settings.wireless_profile);ui.label(format!("PC → Phone: {} × {}, up to {} FPS, {} Mbps",quality.width,quality.height,quality.fps,quality.kbps/1000));ui.label("You can choose a lower profile on your phone. Changes apply to the next connection.");
        if ui.button("Save name and Wi-Fi quality").clicked(){app.apply_wireless_settings();}
        ui.add_space(25.);ui.heading("Trusted Devices");
        if let Some(c)=&app.companion {
            let (mut required,peers)={let store=c.trusted.lock().unwrap();(store.require_approval,store.peers.clone())};
            if ui.checkbox(&mut required,"Require approval for every session").changed(){if let Err(error)=c.trusted.lock().unwrap().set_approval(required){app.status=error;}}
            ui.label("New devices always require QR verification and approval. Forget also disconnects an active session.");
            if peers.is_empty(){ui.label("No remembered devices yet.");}
            for(id,peer)in peers{ui.horizontal(|ui|{ui.label(&peer.name);ui.label(RichText::new(format!("{}…",&id[..12])).small());if ui.button("Forget Device").clicked(){if let Err(error)=c.forget_device(&id){app.status=error;}}});}
        }else{ui.label("Enable wireless connection to manage trusted devices.");if ui.button("Enable wireless").clicked(){app.easy=true;}}
        ui.add_space(25.);ui.heading("Advanced USB / scrcpy quality");ui.add_space(15.);
        ui.add(Slider::new(&mut app.settings.size,720..=2560).text("Maximum edge (pixels)"));
        ui.add(Slider::new(&mut app.settings.fps,15..=60).text("Frame rate"));
        ui.add(Slider::new(&mut app.settings.bitrate,1..=32).text("Bitrate (Mbps)"));
        ui.add_space(20.);ui.checkbox(&mut app.settings.audio,"Device audio");ui.checkbox(&mut app.settings.control,"Mouse and keyboard control");
        ui.checkbox(&mut app.settings.fullscreen,"Start scrcpy in fullscreen");ui.checkbox(&mut app.settings.hardware,"Prefer hardware decoding when scrcpy supports it");
        ui.add_space(25.);ui.heading("Capture folder");ui.add(TextEdit::singleline(&mut app.settings.folder).desired_width(600.));
        ui.add_space(20.);ui.label("Wireless shares your screen and supported device audio without developer settings. Recording and remote control use the advanced scrcpy connection.");
    });
}

fn desktop_session(app:&mut Relay,ctx:&Context){
    CentralPanel::default().frame(Frame::NONE.fill(Color32::from_rgb(246,248,252))).show(ctx,|ui|{
        let area=ui.max_rect();let center=area.center();let active=app.companion.as_ref().is_some_and(|c|c.desktop.active.load(std::sync::atomic::Ordering::Relaxed));
        let panel=Rect::from_center_size(center,vec2(640.,550.));surface(ui,panel,Color32::WHITE);
        self::center(ui,pos2(center.x,panel.top()+33.),if active{"Sharing this computer"}else{"Choose what to share"},26.,INK);
        let peer=app.companion.as_ref().map(|c|c.peer_name.lock().unwrap().clone()).unwrap_or_else(||"Phone".into());self::center(ui,pos2(center.x,panel.top()+78.),&if active{format!("{peer} is viewing the screen or window you selected.")}else{"Select a screen or window in the desktop picker.".into()},14.,SUB);
        illustration(ui,Rect::from_min_max(panel.min+vec2(45.,120.),panel.max-vec2(45.,125.)));
        self::center(ui,pos2(center.x,panel.bottom()-116.),"PC → Phone · View only · Encrypted connection",13.,BLUE);
        if action(ui,Rect::from_center_size(pos2(center.x,panel.bottom()-59.),vec2(236.,44.)),if active{"Stop sharing"}else{"Cancel request"},true){if let Some(c)=&app.companion{c.desktop.stop();}}
    });
}
