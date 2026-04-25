use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct MacTray {
    #[allow(dead_code)]
    tray: TrayIcon,
}

impl MacTray {
    pub fn new(quit_item: MenuItem) -> Result<Self, Box<dyn std::error::Error>> {
        let icon = load_icon();

        let menu = Menu::new();
        menu.append_items(&[&quit_item])?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("EVI 语音输入法")
            .with_icon(icon)
            .build()?;

        Ok(Self { tray })
    }

    pub fn set_state(&self, _state: TrayDisplayState) {}
}

pub enum TrayDisplayState {
    Idle,
    Recording,
    Processing,
}

fn load_icon() -> Icon {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(dir) = exe_dir {
        for name in &["evi.ico", "evi_recording.png"] {
            let path = dir.join(name);
            if path.exists() {
                if let Ok(img) = image::open(&path) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), w, h) {
                        return icon;
                    }
                }
            }
        }
    }
    circle_icon(80, 180, 80)
}

fn circle_icon(r: u8, g: u8, b: u8) -> Icon {
    let size: u32 = 22;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let c = size as f64 / 2.0;
    let rad = c - 2.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - c;
            let dy = y as f64 - c;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= rad {
                let a = if d > rad - 1.0 {
                    ((rad - d + 1.0).min(1.0) * 255.0) as u8
                } else {
                    255
                };
                rgba.extend_from_slice(&[r, g, b, a]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).unwrap()
}
