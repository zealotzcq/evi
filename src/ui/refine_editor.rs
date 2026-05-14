use anyhow::Result;
use std::sync::Arc;

use crate::ui::refine_api::{AuditItem, AuditSubmit, RefineApi, RefineSubmit};

const THANK_YOU_MESSAGES: &[&str] = &[
    "感谢你为系统优化做出的贡献，每一次反馈都让我们变得更好！",
    "太棒了！你的审阅帮助了更多人获得更准确的识别结果。",
    "谢谢你的用心审核，你的参与让语音输入更智能！",
    "又一条优化完成！感谢你与我们一起打磨产品体验。",
    "你的每一份贡献都很珍贵，感谢让系统变得更优秀！",
];

const MAX_DISPLAY: usize = 10;

#[derive(Debug, Clone)]
struct RefineEntry {
    id: i64,
    original: String,
    refined: String,
}

enum Tab {
    Optimize,
    Audit,
}

enum AuditState {
    Idle,
    Loaded(AuditItem),
    ThankYou(String),
    Error(String),
}

struct RefineEditorApp {
    api: Box<dyn RefineApi>,
    uuid: String,
    contribution: i32,

    tab: Tab,
    error_msg: Option<String>,

    entries: Vec<RefineEntry>,
    selected_id: Option<i64>,
    original_display: String,
    edit_text: String,

    audit_state: AuditState,
}

impl RefineEditorApp {
    fn new(api: Box<dyn RefineApi>, uuid: String) -> Self {
        let contribution = match api.get_user_profile(&uuid) {
            Ok(p) => p.contribution,
            Err(e) => {
                log::warn!("获取用户资料失败: {}", e);
                0
            }
        };
        let mut app = Self {
            api,
            uuid,
            contribution,
            tab: Tab::Optimize,
            error_msg: None,
            entries: Vec::new(),
            selected_id: None,
            original_display: String::new(),
            edit_text: String::new(),
            audit_state: AuditState::Idle,
        };
        app.reload();
        app.load_audit_item();
        app
    }

    fn reload(&mut self) {
        self.entries.clear();
        self.selected_id = None;
        self.original_display.clear();
        self.edit_text.clear();
        self.error_msg = None;

        let path = crate::models::refine_db_path();
        match rusqlite::Connection::open(&path) {
            Ok(conn) => {
                let mut stmt = match conn
                    .prepare("SELECT id, original, refined FROM refine_log ORDER BY id DESC")
                {
                    Ok(s) => s,
                    Err(e) => {
                        self.error_msg = Some(format!("查询失败: {}", e));
                        return;
                    }
                };
                let rows = stmt.query_map([], |row| {
                    Ok(RefineEntry {
                        id: row.get(0)?,
                        original: row.get(1)?,
                        refined: row.get(2)?,
                    })
                });
                match rows {
                    Ok(rows) => {
                        for row in rows {
                            match row {
                                Ok(e) => self.entries.push(e),
                                Err(e) => {
                                    self.error_msg = Some(format!("读取记录失败: {}", e));
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("查询失败: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_msg = Some(format!("打开数据库失败: {}", e));
            }
        }
    }

    fn display_entries(&self) -> Vec<&RefineEntry> {
        let mut unrefined: Vec<&RefineEntry> = self
            .entries
            .iter()
            .filter(|e| e.refined.is_empty())
            .collect();
        let mut refined: Vec<&RefineEntry> = self
            .entries
            .iter()
            .filter(|e| !e.refined.is_empty())
            .collect();

        unrefined.truncate(MAX_DISPLAY);
        if unrefined.len() < MAX_DISPLAY {
            let need = MAX_DISPLAY - unrefined.len();
            refined.truncate(need);
            unrefined.extend(refined);
        }
        unrefined
    }

    fn delete_entry(&mut self, id: i64) {
        let path = crate::models::refine_db_path();
        if let Ok(conn) = rusqlite::Connection::open(&path) {
            if conn
                .execute(
                    "DELETE FROM refine_log WHERE id = ?1",
                    rusqlite::params![id],
                )
                .is_ok()
            {
                self.entries.retain(|e| e.id != id);
                if self.selected_id == Some(id) {
                    self.selected_id = None;
                    self.original_display.clear();
                    self.edit_text.clear();
                }
            }
        }
    }

    fn submit_refine(&mut self, id: i64, new_refined: &str) {
        let original = match self.entries.iter().find(|e| e.id == id) {
            Some(e) => e.original.clone(),
            None => return,
        };

        let path = crate::models::refine_db_path();
        if let Ok(conn) = rusqlite::Connection::open(&path) {
            if conn
                .execute(
                    "UPDATE refine_log SET refined = ?1 WHERE id = ?2",
                    rusqlite::params![new_refined, id],
                )
                .is_ok()
            {
                for e in &mut self.entries {
                    if e.id == id {
                        e.refined = new_refined.to_string();
                        break;
                    }
                }
                self.selected_id = None;
                self.original_display.clear();
                self.edit_text.clear();
            }
        }

        match self.api.submit_refine(RefineSubmit {
            uuid: self.uuid.clone(),
            original,
            refined: new_refined.to_string(),
        }) {
            Ok(resp) => {
                if !resp.success {
                    log::warn!("submit_refine API returned failure: {:?}", resp.message);
                }
            }
            Err(e) => {
                log::warn!("submit_refine API error: {}", e);
            }
        }
    }

    fn load_audit_item(&mut self) {
        match self.api.get_pending_audit(&self.uuid) {
            Ok(Some(item)) => {
                self.audit_state = AuditState::Loaded(item);
            }
            Ok(None) => {
                self.audit_state = AuditState::Idle;
            }
            Err(e) => {
                self.audit_state = AuditState::Error(format!("获取待审核内容失败: {}", e));
            }
        }
    }

    fn submit_audit(&mut self, task_id: i64, approved: bool) {
        match self.api.submit_audit(AuditSubmit {
            uuid: self.uuid.clone(),
            task_id,
            approved,
        }) {
            Ok(resp) if resp.success => {
                let idx = std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize
                    % THANK_YOU_MESSAGES.len();
                self.audit_state = AuditState::ThankYou(THANK_YOU_MESSAGES[idx].to_string());
            }
            Ok(resp) => {
                self.audit_state = AuditState::Error(format!(
                    "提交失败: {}",
                    resp.message.unwrap_or_else(|| "未知错误".to_string())
                ));
            }
            Err(e) => {
                self.audit_state = AuditState::Error(format!("提交失败: {}", e));
            }
        }
    }
}

impl eframe::App for RefineEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("体验优化官");
                    ui.label(
                        egui::RichText::new(format!("贡献: {}", self.contribution))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 100, 100)),
                    );
                });
                ui.separator();

                ui.horizontal(|ui| {
                    let opt_active = matches!(self.tab, Tab::Optimize);
                    let audit_active = matches!(self.tab, Tab::Audit);
                    if ui.selectable_label(opt_active, "我的对话").clicked() {
                        self.tab = Tab::Optimize;
                    }
                    if ui.selectable_label(audit_active, "审核助手").clicked() {
                        self.tab = Tab::Audit;
                    }
                });
                ui.separator();

                match self.tab {
                    Tab::Optimize => self.show_optimize_page(ui),
                    Tab::Audit => self.show_audit_page(ui),
                }

                if let Some(ref err) = self.error_msg {
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
        });
    }
}

impl RefineEditorApp {
    fn show_optimize_page(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_height();
        let list_h = (available * 0.50).max(100.0);

        let display: Vec<(i64, String, bool)> = self
            .display_entries()
            .into_iter()
            .map(|e| (e.id, e.original.clone(), e.refined.is_empty()))
            .collect();

        egui::Frame::group(ui.style())
            .inner_margin(8)
            .outer_margin(egui::Margin::same(0))
            .show(ui, |ui| {
                ui.set_min_height(list_h);
                egui::ScrollArea::vertical()
                    .max_height(list_h)
                    .show(ui, |ui| {
                        for (id, original, is_unrefined) in &display {
                            let is_selected = self.selected_id == Some(*id);
                            egui::Frame::group(ui.style())
                                .fill(if is_selected {
                                    egui::Color32::from_rgb(220, 235, 255)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let resp = ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(original).size(14.0),
                                            )
                                            .sense(egui::Sense::click())
                                            .selectable(false),
                                        );
                                        if resp.clicked() && *is_unrefined {
                                            self.selected_id = Some(*id);
                                            self.original_display = original.clone();
                                            self.edit_text = original.clone();
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("X").clicked() {
                                                    self.delete_entry(*id);
                                                }
                                                if !is_unrefined {
                                                    ui.label(
                                                        egui::RichText::new("已提交")
                                                            .color(egui::Color32::from_rgb(
                                                                46, 139, 87,
                                                            ))
                                                            .size(13.0),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                });
                        }
                    });
            });

        ui.add_space(8.0);

        egui::Frame::group(ui.style())
            .inner_margin(10)
            .outer_margin(egui::Margin::same(0))
            .show(ui, |ui| {
                if let Some(id) = self.selected_id {
                    ui.label(egui::RichText::new("识别原文:").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.original_display.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(2)
                            .interactive(false),
                    );

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("优化为:").strong());
                    let edit_resp = ui.add(
                        egui::TextEdit::multiline(&mut self.edit_text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(2),
                    );

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("提交优化").clicked() {
                            let text = self.edit_text.trim().to_string();
                            if !text.is_empty() {
                                self.submit_refine(id, &text);
                            }
                        }
                        if ui.button("取消").clicked() {
                            self.selected_id = None;
                            self.original_display.clear();
                            self.edit_text.clear();
                        }
                        edit_resp.request_focus();
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("选择一条记录进行优化").color(egui::Color32::GRAY),
                        );
                    });
                }
            });
    }

    fn show_audit_page(&mut self, ui: &mut egui::Ui) {
        match &self.audit_state {
            AuditState::Idle => {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("暂无待审核内容").color(egui::Color32::GRAY));
                    ui.add_space(10.0);
                    if ui.button("获取待审核内容").clicked() {
                        self.load_audit_item();
                    }
                });
            }
            AuditState::Loaded(item) => {
                let item = item.clone();
                ui.add_space(8.0);
                egui::Frame::group(ui.style())
                    .inner_margin(10)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("润色前:").strong());
                        ui.add(
                            egui::TextEdit::multiline(&mut item.original.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(3)
                                .interactive(false),
                        );
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("润色后:").strong());
                        ui.add(
                            egui::TextEdit::multiline(&mut item.refined.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(3)
                                .interactive(false),
                        );
                    });

                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("通过").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(46, 139, 87))
                                .min_size(egui::vec2(100.0, 36.0)),
                            )
                            .clicked()
                        {
                            self.submit_audit(item.task_id, true);
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("拒绝").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(200, 60, 60))
                                .min_size(egui::vec2(100.0, 36.0)),
                            )
                            .clicked()
                        {
                            self.submit_audit(item.task_id, false);
                        }
                    });
                });
            }
            AuditState::ThankYou(msg) => {
                let msg = msg.clone();
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(
                        egui::RichText::new("审核已提交")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(46, 139, 87)),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(&msg)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(80, 80, 80)),
                    );
                    ui.add_space(20.0);
                    if ui
                        .add(egui::Button::new("下一条").min_size(egui::vec2(120.0, 32.0)))
                        .clicked()
                    {
                        self.load_audit_item();
                    }
                });
            }
            AuditState::Error(msg) => {
                let msg = msg.clone();
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.colored_label(egui::Color32::RED, &msg);
                    ui.add_space(12.0);
                    if ui.button("重试").clicked() {
                        self.load_audit_item();
                    }
                });
            }
        }
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

pub fn run_refine_editor() -> Result<()> {
    use std::sync::Arc;
    let log_buffer: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    crate::ui::log_capture::init_log_capture(log_buffer);
    let _ = log::set_boxed_logger(Box::new(crate::ui::log_capture::CaptureLogger))
        .map(|()| log::set_max_level(log::LevelFilter::Info));
    log::info!("体验优化官 starting...");

    let uuid = crate::ui::refine_api::get_user_uuid().unwrap_or_default();
    let api = crate::ui::refine_api::create_api();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("体验优化官")
            .with_inner_size([700.0, 560.0])
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        "体验优化官",
        native_options,
        Box::new(move |cc| {
            load_fonts(&cc.egui_ctx);
            Ok(Box::new(RefineEditorApp::new(api, uuid)))
        }),
    );
    match &result {
        Ok(()) => log::info!("体验优化官 closed normally"),
        Err(e) => log::error!("体验优化官 error: {}", e),
    }
    result.map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}
