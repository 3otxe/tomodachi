
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, CheckMenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};
use tracing::info;

pub struct TrayMenuIds {
    pub status_id: tray_icon::menu::MenuId,
    pub quit_id: tray_icon::menu::MenuId,
    pub movable_id: tray_icon::menu::MenuId,
    pub op_25_id: tray_icon::menu::MenuId,
    pub op_50_id: tray_icon::menu::MenuId,
    pub op_75_id: tray_icon::menu::MenuId,
    pub op_100_id: tray_icon::menu::MenuId,
}

pub fn create_tray() -> anyhow::Result<(TrayIcon, TrayMenuIds)> {
    let menu = Menu::new();

    let status_item = MenuItem::new("🐾 Tomodachi Status", true, None);
    let status_id = status_item.id().clone();
    
    let settings_menu = Submenu::new("Settings", true);
    
    let movable_item = CheckMenuItem::new("Movable", true, false, None);
    let movable_id = movable_item.id().clone();
    
    let opacity_menu = Submenu::new("Opacity", true);
    
    let op_25 = MenuItem::new("25%", true, None);
    let op_25_id = op_25.id().clone();
    
    let op_50 = MenuItem::new("50%", true, None);
    let op_50_id = op_50.id().clone();
    
    let op_75 = MenuItem::new("75%", true, None);
    let op_75_id = op_75.id().clone();
    
    let op_100 = MenuItem::new("100%", true, None);
    let op_100_id = op_100.id().clone();
    
    opacity_menu.append(&op_25)?;
    opacity_menu.append(&op_50)?;
    opacity_menu.append(&op_75)?;
    opacity_menu.append(&op_100)?;
    
    settings_menu.append(&movable_item)?;
    settings_menu.append(&opacity_menu)?;

    let quit_item = MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();

    menu.append(&status_item)?;
    menu.append(&settings_menu)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let icon = create_icon()?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Tomodachi — your terminal companion")
        .with_icon(icon)
        .build()?;

    info!("system tray icon created");

    Ok((tray, TrayMenuIds { status_id, quit_id, movable_id, op_25_id, op_50_id, op_75_id, op_100_id }))
}

pub fn poll_tray_event() -> Option<tray_icon::menu::MenuId> {
    MenuEvent::receiver().try_recv().ok().map(|e| e.id().clone())
}

fn create_icon() -> anyhow::Result<tray_icon::Icon> {
    
    let size = crate::sprite::WINDOW_SIZE;
    let pixels_u32 = crate::sprite::render_sprite(tomodachi_shared::Mood::Happy, 0);
    
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for pixel in pixels_u32 {
        
        let a = ((pixel >> 24) & 0xFF) as u8;
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        
        rgba.push(r);
        rgba.push(g);
        rgba.push(b);
        rgba.push(a);
    }

    let icon = tray_icon::Icon::from_rgba(rgba, size, size)?;
    Ok(icon)
}
