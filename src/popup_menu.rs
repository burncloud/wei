use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent},
    TrayIcon, TrayIconBuilder,
};
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
pub struct PopupMenu {
    _tray_icon: TrayIcon,
}

#[allow(dead_code)]
impl PopupMenu {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Wei System")
            .build()?;

        Ok(PopupMenu { _tray_icon: tray_icon })
    }
}

// 创建默认菜单的便捷函数
pub fn create_default_tray_menu() -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new();

    // 添加菜单项
    let show_item = MenuItem::new("显示界面", true, None);
    let files_item = MenuItem::new("文件管理", true, None);
    let sysinfo_item = MenuItem::new("系统信息", true, None);
    let network_item = MenuItem::new("网络状态", true, None);
    let settings_item = MenuItem::new("设置", true, None);
    let help_item = MenuItem::new("帮助", true, None);
    let quit_item = MenuItem::new("退出", true, None);

    menu.append(&show_item)?;
    menu.append(&files_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&sysinfo_item)?;
    menu.append(&network_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&settings_item)?;
    menu.append(&help_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let icon = create_default_icon();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Wei System")
        .with_icon(icon)
        .build()?;

    // 处理菜单事件
    let menu_channel = MenuEvent::receiver();

    loop {
        if let Ok(event) = menu_channel.try_recv() {
            handle_menu_event(&event);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

// 处理菜单点击事件
fn handle_menu_event(event: &MenuEvent) {
    let item_title = event.id.0.as_str(); // 获取菜单项标题

    match item_title {
        id if id.contains("显示界面") => {
            println!("显示界面");
            if let Err(e) = webbrowser::open("http://127.0.0.1:1115") {
                println!("打开浏览器失败: {}", e);
            }
        }
        id if id.contains("文件管理") => {
            println!("文件管理");
            open_file_manager();
        }
        id if id.contains("系统信息") => {
            println!("系统信息");
            show_system_info();
        }
        id if id.contains("网络状态") => {
            println!("网络状态检查");
        }
        id if id.contains("设置") => {
            println!("打开设置");
        }
        id if id.contains("帮助") => {
            println!("打开帮助");
            if let Err(e) = webbrowser::open("https://github.com/zuiyue-com/wei") {
                println!("打开帮助页面失败: {}", e);
            }
        }
        id if id.contains("退出") => {
            println!("退出应用");
            wei_env::stop();
            std::process::exit(0);
        }
        _ => {
            println!("未知菜单项: {}", item_title);
        }
    }
}

fn open_file_manager() {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").spawn().ok();
    #[cfg(target_os = "linux")]
    std::process::Command::new("nautilus").spawn().ok();
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(".").spawn().ok();
}

fn show_system_info() {
    println!("操作系统: {}", std::env::consts::OS);
    println!("架构: {}", std::env::consts::ARCH);

    #[cfg(target_os = "windows")]
    {
        use tauri_winrt_notification::{Duration, Sound, Toast};
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title("系统信息")
            .text1(&format!("OS: {} | 架构: {}", std::env::consts::OS, std::env::consts::ARCH))
            .sound(Some(Sound::Default))
            .duration(Duration::Short)
            .show();
    }
}

// 创建默认图标
fn create_default_icon() -> tray_icon::Icon {
    // 创建一个简单的16x16像素的RGBA图标
    let size = 16;
    let mut rgba = Vec::with_capacity(size * size * 4);

    for y in 0..size {
        for x in 0..size {
            // 创建一个简单的渐变图标
            let r = ((x as f32 / size as f32) * 255.0) as u8;
            let g = ((y as f32 / size as f32) * 255.0) as u8;
            let b = 150u8;
            let a = 255u8;
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }

    tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).unwrap_or_else(|_| {
        // 如果创建失败，创建一个纯色图标
        let mut rgba = Vec::with_capacity(size * size * 4);
        for _ in 0..(size * size) {
            rgba.extend_from_slice(&[100, 150, 200, 255]);
        }
        tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).unwrap()
    })
}