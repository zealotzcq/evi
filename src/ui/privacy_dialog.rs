use std::sync::Arc;

const PRIVACY_TEXT: &str = "\
EVI 语音输入法 — 隐私保护承诺

• 对话记录仅保存在本地，不会自动上传
• 你可以随时查看、删除任意一条记录
• 上传由你手动选择，仅提交你确认的条目
• 通过匿名随机序列标识用户身份，不收集任何个人信息
• 数据传输全程加密，服务器仅存储提交的内容";

struct PrivacyDialog {
    dont_show: bool,
    closed: bool,
}

impl PrivacyDialog {
    fn new() -> Self {
        let ack = crate::Config::load()
            .map(|c| c.privacy_acknowledged)
            .unwrap_or(false);
        Self {
            dont_show: ack,
            closed: false,
        }
    }
}

impl eframe::App for PrivacyDialog {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.closed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("隐私保护说明");
            });
            ui.add_space(6.0);

            egui::Frame::group(ui.style())
                .inner_margin(10)
                .fill(egui::Color32::from_rgb(245, 255, 245))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(PRIVACY_TEXT)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(40, 40, 40)),
                    );
                });

            ui.add_space(6.0);
            ui.checkbox(&mut self.dont_show, "启动时不再显示此消息");
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("我知道了")
                                .size(14.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(46, 139, 87))
                        .min_size(egui::vec2(100.0, 28.0)),
                    )
                    .clicked()
                {
                    if let Err(e) = crate::Config::save_privacy_acknowledged(self.dont_show) {
                        log::warn!("Failed to save privacy_acknowledged: {}", e);
                    }
                    self.closed = true;
                }
            });
        });
    }
}

fn load_fonts(ctx: &egui::Context) {
    let paths: &[&str] = &[
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("chinese".into(), Arc::new(egui::FontData::from_owned(data)));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".into());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese".into());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

pub fn run_privacy_dialog() -> anyhow::Result<()> {
    let log_buffer: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    crate::ui::log_capture::init_log_capture(log_buffer);
    let _ = log::set_boxed_logger(Box::new(crate::ui::log_capture::CaptureLogger))
        .map(|()| log::set_max_level(log::LevelFilter::Info));

    let native_options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_title("隐私保护说明")
            .with_inner_size([400.0, 310.0])
            .with_min_inner_size([400.0, 310.0])
            .with_max_inner_size([400.0, 310.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "隐私保护说明",
        native_options,
        Box::new(move |cc| {
            load_fonts(&cc.egui_ctx);
            Ok(Box::new(PrivacyDialog::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}
