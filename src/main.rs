#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rfd::FileDialog;
use std::time::{Duration, Instant};
use eframe::{App, egui, Frame, epaint::Stroke};
use egui::{Align, Color32, Label, Layout, TextStyle};
use rusqlite::{params, Connection, Result};

struct MyApp {
    paper_id: String,
    scores: [u8; 5],
    db_connection: Connection,
    dimensions: [String; 5],
    show_submission_success: bool,
    success_message_time: Instant,
    show_input_error: bool,
    error_message_time: Instant,
    submission_records: Vec<(u8, [u8; 5])>, // 用来存储提交记录的向量
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {

// 创建字体定义
        let mut fonts = egui::FontDefinitions::default();

        // 添加自定义字体
        fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_static(include_bytes!("/Users/kawaroii/RustRover/paper_review_client/Alibaba-PuHuiTi-Regular.subset.otf")),
        );

        // 定义字体家族中包含的字体
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());

        // 设置默认的字体大小和家族
        cc.egui_ctx.set_fonts(fonts.clone());
        cc.egui_ctx.set_style(egui::Style {
            text_styles: std::collections::BTreeMap::from([
                (TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
                (TextStyle::Monospace, egui::FontId::new(16.0, egui::FontFamily::Monospace)),
                (TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
                (TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
                (TextStyle::Small, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
            ]),
            visuals: egui::Visuals {
                ..egui::Visuals::light()
            },
            ..Default::default()
        });
        let mut visuals = cc.egui_ctx.style().visuals.clone();
        visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(40, 40, 40);
        visuals.window_stroke.width = 2.0;
        cc.egui_ctx.set_visuals(visuals);

        if let Some(ss) = FileDialog::new().pick_folder(){
            println!("{}",ss.display());
        }

        let mut db_path = std::env::current_exe().expect("Failed to find current executable path");

        db_path.pop();
        #[cfg(target_os = "macos")]
        {
            db_path.pop();
            db_path.pop();
            db_path.pop();
        }

        db_path.push("papers_scores.db");

        let db_path = db_path.to_str().expect("Path is not valid unicode").to_string();
        println!("Database file path: {:?}", db_path);

        let connection = Connection::open(db_path).expect("Failed to connect to DB");
        println!("Connected to database successfully. Database path: {:?}", connection.path().unwrap());

        Self::create_table_if_not_exists(&connection).expect("Failed to create table");
        // 使用 CreationContext 初始化你的应用
        Self {
            paper_id: String::new(),
            scores: [0; 5],
            db_connection: connection,
            dimensions: ["TBD".to_string(), "TBD".to_string(), "TBD".to_string(), "TBD".to_string(), "TBD".to_string()],
            show_submission_success: false,
            success_message_time: Instant::now(),
            show_input_error: false,
            error_message_time: Instant::now(),
            submission_records: Vec::new(), // 初始化提交记录向量
        }
    }
    fn create_table_if_not_exists(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS papers (
                id INTEGER PRIMARY KEY,
                score1 INTEGER NOT NULL,
                score2 INTEGER NOT NULL,
                score3 INTEGER NOT NULL,
                score4 INTEGER NOT NULL,
                score5 INTEGER NOT NULL
             )",
            [],
        )?;
        Ok(())
    }



    fn update_or_insert_paper(&mut self) -> Result<()> {
        // 清除上一次的错误状态
        self.show_input_error = false;
        // 验证输入是否为1到150之间的数字
        // let paper_id = self.paper_id.clone();
        match self.paper_id.parse::<i32>() {
            Ok(number) if (1..=150).contains(&number) => {
                // 如果是有效数字，则继续之前的insert或update逻辑
                self.db_connection.execute(
                    "INSERT OR REPLACE INTO papers (id, score1, score2, score3, score4, score5)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        number,
                        self.scores[0],
                        self.scores[1],
                        self.scores[2],
                        self.scores[3],
                        self.scores[4]
                    ],
                )?;
                self.show_submission_success = true;
                self.paper_id.clear();
                self.scores = [0;5];
                self.success_message_time = Instant::now();
                Ok(())
            },
            _ => {
                // 如果输入无效，设置错误状态并返回错误
                self.show_input_error = true;
                self.error_message_time = Instant::now();
                Err(rusqlite::Error::InvalidQuery)
            }
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        {
            let r_stmt = self.db_connection.prepare("SELECT id, score1, score2, score3, score4, score5 FROM papers");
            let mut stmt = match r_stmt {
                Ok(sttmt) => sttmt,
                Err(_) => {
                    return;
                }
            };
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get(0)?,
                    [
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ]
                ))
            });
            self.submission_records = Vec::new();
            for row in rows.unwrap() {
                let row: (u8, [u8; 5]) = row.unwrap();

                self.submission_records.push(row);
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Color32::from_rgb(180, 195, 200),
                inner_margin: egui::Vec2::new(10.0, 10.0).into(), // 设置内边距
                stroke: Stroke::new(1.0, Color32::from_rgb(180, 195, 200)),
                ..Default::default()
            })
            .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::Min).with_cross_align(Align::Min), |ui| {
                    ui.heading("社科杯论文评分系统");
                });
                ui.with_layout(Layout::right_to_left(Align::Min).with_cross_align(Align::Min), |ui| {
                    let mut exit_button_style = (*ctx.style()).clone();
                    exit_button_style.visuals.widgets.hovered.bg_fill =
                        Color32::from_rgb(220, 150, 120); // 红色背景
                    let exit_button = egui::Button::new("退出")
                        .fill(exit_button_style.visuals.widgets.hovered.bg_fill); // 使用自定义颜色
                    let button_size = egui::Vec2::new(50.0, 20.0);

                    if ui.add_sized(button_size, exit_button).clicked() {
                        std::process::exit(0);
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label("论文编号 (1-150):");
                ui.with_layout(Layout::left_to_right(Align::Min).with_cross_align(Align::Min), |ui| {
                    // ui.add(Label::new("REQUEST NAME: ").wrap(true));
                    ui.style_mut().visuals.extreme_bg_color = Color32::from_rgb(235, 240, 240);
                    ui.text_edit_singleline(&mut self.paper_id);
                });
            });

            ui.separator();
            for (i, score) in self.scores.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}", self.dimensions[i]));
                    ui.with_layout(Layout::left_to_right(Align::Min).with_cross_align(Align::Min), |ui| {
                        ui.style_mut().visuals.extreme_bg_color = Color32::from_rgb(235, 240, 240);
                        ui.add(egui::Slider::new(score, 0..=10).min_decimals(0).max_decimals(1).step_by(0.5));
                        ui.style_mut().visuals.extreme_bg_color = Color32::from_rgb(235, 240, 240);

                    });
                });
            }

            let mut submit_button_style = (*ctx.style()).clone();

            submit_button_style.visuals.widgets.inactive.bg_fill =
                Color32::from_rgb(110, 150, 140);
            submit_button_style.visuals.widgets.inactive.fg_stroke.color =
                Color32::from_rgb(230, 240, 250);

            let button_size = egui::Vec2::new(100.0, 40.0);

            ui.horizontal(|ui| {
                let submit_button = egui::Button::new("提交")
                    .fill(submit_button_style.visuals.widgets.inactive.bg_fill)
                    .stroke(Stroke::new(1.0, submit_button_style.visuals.widgets.inactive.fg_stroke.color));
                if ui.add_sized(button_size, submit_button).clicked() {
                    if let Err(err) = self.update_or_insert_paper() {
                        eprintln!("Error submitting paper data: {}", err);
                    }
                }
            });
            ui.add(Label::new("\t").wrap(true));

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("提交记录"); // 表格标题
                ui.separator(); // 分隔线

                // 表头
                ui.horizontal(|ui| {
                    // 使用add_sized方法设置固定宽度
                    ui.add_sized([100.0, ui.text_style_height(&TextStyle::Body)], Label::new("论文编号"));
                    for i in 1..=5 {
                        // 设置固定宽度以对齐
                        ui.add_sized([100.0, ui.text_style_height(&TextStyle::Body)],
                                     Label::new(format!("\tScore {}", i)));
                    }
                });
                ui.separator(); // 表头下的分隔线

                // 遍历 submission_records 并渲染每一行
                for (paper_id, scores) in &self.submission_records {
                    ui.horizontal(|ui| {
                        // 设置固定宽度以对齐
                        ui.add_sized([100.0, ui.text_style_height(&TextStyle::Body)],
                                     Label::new(format!("  {}", paper_id)));
                        for &score in scores {
                            // 设置固定宽度以对齐
                            ui.add_sized([100.0, ui.text_style_height(&TextStyle::Body)],
                                         Label::new(format!("  {:.2}", score)));
                        }
                    });
                }
            });

        });

        if self.show_submission_success && self.success_message_time.elapsed() > Duration::from_secs(1) {
            self.show_submission_success = false; // 关闭消息
        }

        if self.show_submission_success {
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    //fill: Color32::from_rgb(230, 240, 255), // 背景颜色
                    inner_margin: egui::Vec2::new(10.0, 10.0).into(), // 设置内边距
                    ..Default::default()
                })
                .show(ctx, |ui| {
                let available_size = ui.available_size();
                egui::Window::new("Success")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .fixed_pos([available_size.x / 2.0, available_size.y / 2.0])
                    .title_bar(false)
                    .show(ui.ctx(), |ui| {
                        ui.heading("提交成功!");
                    });
            });
        }

        // 如果需要显示错误消息
        if self.show_input_error && self.error_message_time.elapsed() > Duration::from_secs(1) {
            self.show_input_error = false; // 关闭消息
        }

        if self.show_input_error {
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    //fill: Color32::from_rgb(230, 240, 255), // 背景颜色
                    inner_margin: egui::Vec2::new(10.0, 10.0).into(), // 设置内边距
                    ..Default::default()
                })
                .show(ctx, |ui| {
                let available_size = ui.available_size();
                egui::Window::new("Error")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .fixed_pos([available_size.x / 2.0, available_size.y / 2.0])
                    .title_bar(false)
                    .show(ui.ctx(), |ui| {
                        ui.heading("请正确输入论文编号（1-150）");
                    });
            });
        }
    }

}


fn main() {
    // let options = eframe::NativeOptions::default();
    let mut options = eframe::NativeOptions::default();
    options.default_theme = eframe::Theme::Light;
    options.viewport.min_inner_size = Some(egui::vec2(700.0, 500.0));
    options.centered = true;
    eframe::run_native("PaperEvolution", options, Box::new(|cc| Box::new(MyApp::new(cc)))).expect("TODO: panic message");
}