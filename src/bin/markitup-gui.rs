use eframe::{egui};
use markitup;
use std::path::PathBuf;
use rfd::FileDialog;
use pulldown_cmark::{Parser,Options};
use egui_commonmark::CommonMarkViewer;
use std::thread;
use std::sync::{Arc,Mutex};
use crossbeam_channel::{unbounded, Sender, Receiver}; // 引入 crossbeam_channel
use regex::Regex;
use markitup::config;

#[derive(Debug,PartialEq,Clone)]
enum ConvertState{
    Idle,
    Converting(String),
    Down(String),
    Error(String),
}


impl Default for ConvertState{
    fn default() -> Self{
        ConvertState:: Idle
    }
}

#[derive(Debug)]
enum RightPanelMode{
    Preview,
    Editor,
}

impl Default for RightPanelMode{
    fn default()->Self{
        RightPanelMode::Preview
    }
}
enum WorkerMessage {
    ConversionResult {
        full_markdown: String,   // 完整的 Markdown 内容
        display_markdown: String, // 经过 Base64 替换后的 Markdown 内容，用于编辑器显示
    },
    Error(String), // 转换过程中发生的错误
}

fn replace_base64_in_markdown(markdown:&str) ->String{
    let re = Regex::new(r"\((data:image/[^;]+;base64,[^)]+)\)").unwrap();
    re.replace_all(markdown, "(base64_image_placeholder)").into_owned()
}

pub struct UIFramework{
    show_config_panel:bool,
    show_help_panel:bool,
    
    file_list: Vec<PathBuf>,
    select_file_path: Option<PathBuf>,
    current_markdown_content: String,
    pub editor_display_content: String, 

    right_panel_mode: RightPanelMode,
    markdown_cache:egui_commonmark::CommonMarkCache,

    //window sytle
    pub font_size_heading :f32,
    pub font_size_body:f32,
    pub background_color: egui::Color32,
    pub text_color: egui::Color32,
    
    //convert state
    convert_state: Arc<Mutex<ConvertState>>,
    pub egui_ctx: egui::Context,
    pub worker_sender: Sender<WorkerMessage>,   // 发送给工作线程 (通常不会从UI发送，但Default需要初始化)
    pub worker_receiver: Receiver<WorkerMessage>,
    
    //config
    pub config_first_input: Option<String>,
    pub config_second_input: Option<String>,
    pub config_choice: bool,
}
impl Default for UIFramework{

    
    fn default()->Self{
        let (tx, rx) = unbounded();
        Self{
            show_config_panel:false,
            show_help_panel:false,

            file_list:Vec::new(),
            select_file_path:None,
            current_markdown_content: String::new(),
            editor_display_content: String::new(),

            right_panel_mode: RightPanelMode::default(),
            markdown_cache: egui_commonmark::CommonMarkCache::default(),

            font_size_heading:22.0,
            font_size_body:18.0,
            background_color:egui::Color32::from_rgb(60, 55, 50),
            text_color: egui::Color32::from_rgb(240,240,230),
            convert_state: Arc::new(Mutex::new(ConvertState::Idle)), 
            egui_ctx: egui::Context::default(),

            worker_sender: tx,
            worker_receiver: rx,
            
            config_first_input: None, // 填空题1的默认值
            config_second_input: None, // 填空题2的默认值
            config_choice: false,
        }

    }
}

impl eframe::App for UIFramework{
    fn update(&mut self, ctx: &egui::Context, _frame:&mut eframe::Frame){
        let mut clicked_file_path: Option<PathBuf> = None;
        while let Ok(msg) = self.worker_receiver.try_recv() {
            let mut state_guard = self.convert_state.lock().unwrap(); // 获取转换状态的锁
            match msg {
                WorkerMessage::ConversionResult { full_markdown, display_markdown } => {
                    // 如果收到了成功转换的消息
                    self.current_markdown_content = full_markdown; // 更新完整 Markdown 内容
                    self.editor_display_content = display_markdown; // 更新编辑器显示内容
                    *state_guard = ConvertState::Idle; // 转换完成，将状态重置为 Idle
                    // 注意：这里将状态重置为 Idle，以便在下一次更新中可以显示最终内容，
                    // 而不是一直显示 "Done" 状态。
                }
                WorkerMessage::Error(msg) => {
                    *state_guard = ConvertState::Error(msg); // 更新状态为错误
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx,|ui|{
            ui.horizontal(|ui|{
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP),|ui|{
                    ui.menu_button("file",|ui_file_menu|{
                        if ui_file_menu.button("Open").clicked(){
                           self.open_files_dialog(); 
                        }
                    });

                    if ui.button("config").clicked() {
                        println!("点击了 config 按钮");
                        self.show_config_panel=!self.show_config_panel;
                        self.show_help_panel=false;
                    }
                    
                });//left_to_right end
                    
  
             });//horizontal end
        });//topbottom end
        egui::CentralPanel::default().show(ctx,|ui|{
            egui::SidePanel::left("file_list").exact_width(200.0).show_inside(ui,|ui|{
                ui.vertical_centered(|ui| { // 让按钮居中
                    ui.add_space(10.0); // 顶部间距
                    ui.heading("file list");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut indices_to_remove = Vec::new();
                        
                        for (idx, path_buf) in self.file_list.iter().enumerate() {
                            let file_name = path_buf.file_name().unwrap_or_default().to_string_lossy();
                            let is_selected = self.select_file_path.as_ref() == Some(path_buf);

                            ui.horizontal(|ui| {
                                // 文件名标签 (可选中)
                                let response = if is_selected {
                                    ui.selectable_label(true, file_name.as_ref())
                                } else {
                                    ui.selectable_label(false, file_name.as_ref())
                                };

                                if response.clicked() {
                                    if !is_selected {
                                        clicked_file_path = Some(path_buf.clone()); // 克隆路径并存储
                                    }
                                }

                                // 添加删除按钮 (靠右对齐)
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let delete_response = ui.add(
                                        egui::Button::new("❌")
                                            .small()
                                            .frame(false)
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::NONE)
                                            
                                    )
                                    .on_hover_text("从列表移除");

                                    if delete_response.clicked() {
                                        indices_to_remove.push(idx); // 记录索引
                                    }
                                });
                            }); // end horizontal for each file item
                        } // end for loop

                        // 在遍历结束后，从后往前删除元素以避免索引问题
                        for &idx in indices_to_remove.iter().rev() {
                            let removed_path = self.file_list.remove(idx);
                            println!("Removed file from list: {:?}", removed_path.file_name().unwrap_or_default());

                            // 如果被移除的是当前选中的文件，则清除相关状态
                            if self.select_file_path.as_ref() == Some(&removed_path) {
                                self.select_file_path = None;
                                self.current_markdown_content.clear();
                                self.editor_display_content.clear();
                                *self.convert_state.lock().unwrap() = ConvertState::Idle; // 重置转换状态
                            }
                        }

                        // 如果文件列表变空了，且之前有选中文件，确保状态被清除
                        if self.file_list.is_empty() && self.select_file_path.is_some() {
                             self.select_file_path = None;
                             self.current_markdown_content.clear();
                             self.editor_display_content.clear();
                             *self.convert_state.lock().unwrap() = ConvertState::Idle;
                        }
                        }); // end scroll area;

                    });//end vertical_centered
            });//end left side panel
            if let Some(path_to_load) = clicked_file_path {
                self.load_and_set_markdown_content(&path_to_load);
            }
            egui::CentralPanel::default().show_inside(ui,|ui|{
                ui.vertical(|ui|{
                    ui.heading(match self.right_panel_mode{
                        RightPanelMode::Preview => "Markdown preview",
                        RightPanelMode::Editor=>"Markdown Editor(Source Code)",
                    });
                    ui.add_space(5.0);

                    ui.horizontal(|ui|{
                        if ui
                            .button(match self.right_panel_mode {
                                RightPanelMode::Preview => "Change to Editor Mode",
                                RightPanelMode::Editor => "Change to Preview Mode",
                            })
                            .clicked()
                        {
                            self.right_panel_mode = match self.right_panel_mode {
                                RightPanelMode::Preview => RightPanelMode::Editor,
                                RightPanelMode::Editor => RightPanelMode::Preview,
                            };
                            println!("当前模式: {:?}", self.right_panel_mode);
                        }
                        ui.add_space(10.0); // 按钮之间的间距
                        if ui.button("Save Markdown").clicked(){
                            self.save_markdown_content();
                        }
                    });//end horizontal
                    ui.separator();
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical().show(ui,|ui|{
                        match self.right_panel_mode{
                            RightPanelMode::Preview =>{
                                let available_size = ui.available_size();
                                egui::Frame::none()
                                    .show(ui,|frame_ui|{
                                        let viewer = CommonMarkViewer::new("markdown_viewer_unique_id");
                                        viewer.show(frame_ui, &mut self.markdown_cache, &self.current_markdown_content);
                                    });
                            }
                            RightPanelMode::Editor =>{
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.current_markdown_content)
                                        .desired_width(f32::INFINITY) // 宽度填充可用空间
                                        .desired_rows(20) // 默认高度（行数）
                                      );
                            }
                        }
                    });//end scrollarea

                });//end vertical

            });//end central panel

       }); //end central panel

        if self.show_config_panel{
        egui::Window::new("config")
            .open(&mut self.show_config_panel)
            .show(ctx,|ui|{
                ui.heading("config");
                ui.vertical(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                        ui.label("deepseek api key");
                        let mut text_value = self.config_first_input.get_or_insert_with(String::new);

                            ui.text_edit_singleline(text_value);

                            if text_value.is_empty() {
                                self.config_first_input = None;
                            }

                    });
                    ui.vertical(|ui| {
                        ui.label("doubao api key");
                        let mut text_value = self.config_second_input.get_or_insert_with(String::new);

                            ui.text_edit_singleline(text_value);

                            if text_value.is_empty() {
                                self.config_second_input = None;
                            }
                    });
                    ui.add_space(5.0);

                    // --- 选择题 ---
                    ui.horizontal(|ui| {
                        ui.label("Enable AI?");
                        // egui::RadioButton 是一个很好的选择题组件
                        // 第一个选项
                        ui.radio_value(&mut self.config_choice, false, "No");
                        // 第二个选项
                        ui.radio_value(&mut self.config_choice, true, "Yes");
                    });
                    ui.add_space(10.0);

                    ui.separator();
                    ui.add_space(10.0);

                    if ui.button("Apply Settings").clicked(){
                       println!("asfawef");
                       // config::set_is_ai_enpower(self.config_choice.clone());
                       // config::set_deepseek_api_key(self.config_first_input.clone());
                       // config::set_doubao_api_key(self.config_second_input.clone());
                    }
            });
        });
        }
        if self.show_help_panel{
            egui::Window::new("help")
                .open(&mut self.show_help_panel)
                .show(ctx,|ui|{
                    ui.heading("help");
                    ui.add_space(10.0);
                });
        }

    }
}
pub fn createFrame(){
    let app_name = "Markitup"; 

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1500.0, 1200.0]) // 初始窗口大小
            .with_min_inner_size([300.0, 200.0]) // 最小窗口大小
            .with_title(app_name),
        vsync:true,
        multisampling:4,
        ..Default::default()

    };
    let app=UIFramework::default();
    eframe::run_native(
        app_name,
        native_options,
        Box::new(|cc| Box::new(UIFramework::new(cc))),
        );
}
impl UIFramework{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default(); 
        app.egui_ctx = cc.egui_ctx.clone();
        let mut fonts= egui::FontDefinitions::default();
        fonts.font_data.insert(
            "my_custom_font".to_owned(), // Give your font a unique name within egui
            egui::FontData::from_static(include_bytes!("../../font.ttf")), // Adjust path as needed
        );
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "my_custom_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "my_custom_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(app.font_size_heading)); // 使用标题字号作为按钮字号
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(app.font_size_body));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(app.font_size_heading));

        // 设置颜色
        style.visuals.window_fill = app.background_color;
        style.visuals.panel_fill = app.background_color;
        //style.visuals.text_color = app.text_color; // 默认文本颜色

        cc.egui_ctx.set_style(style);

        cc.egui_ctx.set_pixels_per_point(1.2);
        app
    }

    fn open_files_dialog(&mut self) {
        let result = FileDialog::new()
            .set_title("Select files")
            .add_filter("All Files", &["*"])
            .pick_files(); // This call is blocking

        if let Some(paths) = result {
            for path_buf in paths {
                if !self.file_list.contains(&path_buf) { // Avoid duplicates
                    self.file_list.push(path_buf.clone());
                    println!("Added file: {:?}", path_buf);
                }
            }
        } else {
            println!("File selection canceled");
        }
    }
    fn save_markdown_content(&self){
        if let Some(ref selected_path) = self.select_file_path {
            // 建议保存为 .md 文件，并尝试使用原始文件的目录和文件名
            let default_save_path = selected_path.with_extension("md");
            let current_dir_path = PathBuf::from(".");
            let file_dialog_result = FileDialog::new()
                .set_title("另存为 Markdown...")
                .add_filter("Markdown 文件", &["md"])
                // 设置默认目录为当前选定文件的父目录，如果文件没有父目录，则使用当前工作目录
                .set_directory(&current_dir_path)
                .set_file_name(default_save_path.file_name().unwrap_or_default().to_string_lossy())
                .save_file(); // 这会阻塞当前线程直到用户选择或取消

            if let Some(save_path) = file_dialog_result {
                match std::fs::write(&save_path, &self.current_markdown_content) {
                    Ok(_) => println!("Markdown 已成功保存到: {:?}", save_path),
                    Err(e) => eprintln!("保存 Markdown 失败: {}", e),
                }
            } else {
                println!("保存操作已取消。");
            }
        } else {
            println!("没有文件被选中，无法保存内容。");
            // add ui?
        }
    }
    pub fn load_and_set_markdown_content(&mut self, path_buf: &PathBuf) {
        self.select_file_path = Some(path_buf.clone());
        let file_name_str = path_buf.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned();

        // 2. 将转换状态设置为 "Converting"，以便 UI 可以显示加载提示
        *self.convert_state.lock().unwrap() = ConvertState::Converting(file_name_str.clone());

        // 3. 克隆必要的变量以发送到新线程
        let ui_ctx = self.egui_ctx.clone(); // egui context 用于请求 UI 重绘
        let convert_state_arc = Arc::clone(&self.convert_state); // 共享转换状态
        let path_for_thread = path_buf.clone(); // 要转换的文件路径
        let sender_for_thread = self.worker_sender.clone(); // 用于将结果发送回主线程

        // 4. 启动一个新线程来执行耗时操作
        thread::spawn(move || {
            // 尝试将 PathBuf 转换为 &str，如果失败则返回错误
            let result = if let Some(path_str) = path_for_thread.to_str() {
                // 调用您的 markitup 库进行转换
                markitup::convert_from_path(path_str)
            } else {
                Err(format!("文件路径包含无效的 UTF-8 字符: {}", path_for_thread.display()))
            };

            match result {
                Ok(full_markdown_content) => {
                    let display_content = replace_base64_in_markdown(&full_markdown_content);
                    sender_for_thread.send(WorkerMessage::ConversionResult {
                        full_markdown: full_markdown_content,
                        display_markdown: display_content,
                    }).unwrap(); 
                },
                Err(e) => {
                    sender_for_thread.send(WorkerMessage::Error(format!("转换文件 '{}' 失败: {}", file_name_str, e))).unwrap();
                },
            }
            ui_ctx.request_repaint();
        });
    }
    
}

fn main(){
    createFrame();

}
