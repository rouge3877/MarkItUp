use eframe::{egui};
use markitup;
use std::path::PathBuf;
use rfd::FileDialog;
use pulldown_cmark::{Parser,Options};
use egui_commonmark::CommonMarkViewer;

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

#[derive(Default)]
pub struct UIFramework{
    show_config_panel:bool,
    show_help_panel:bool,
    

    editor_markdown_content:String,
    is_file_dialog_open:bool,
    
    file_list: Vec<PathBuf>,
    select_file_path: Option<PathBuf>,
    current_markdown_content: String,
    right_panel_mode: RightPanelMode,
    markdown_cache:egui_commonmark::CommonMarkCache,
}

impl eframe::App for UIFramework{
    fn update(&mut self, ctx: &egui::Context, _frame:&mut eframe::Frame){
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
                    if ui.button("help").clicked() {
                        println!("点击了 help 按钮");
                        self.show_help_panel=!self.show_help_panel;
                        self.show_config_panel=false;
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
                        let file_list_guard = &self.file_list;
                        if file_list_guard.is_empty() {
                            ui.label("Please add files.");
                        } else {
                        let mut selected_idx = -1; // 用来记录哪个文件被选中
                            for (idx, path_buf) in self.file_list.iter().enumerate() {
                                let file_name = path_buf.file_name().unwrap_or_default().to_string_lossy();
                                let is_selected = self.select_file_path.as_ref() == Some(path_buf);

                                let response = ui.selectable_value(&mut is_selected.clone(), true, file_name);

                                if response.clicked() {
                                    self.select_file_path = Some(path_buf.clone());
                                    if let Some(path) = &self.select_file_path {
                                        if let Some(path_str) = path.to_str(){
                                            dbg!(path_str);
                                            let content= markitup::convert_from_path(path_str);
                                            match content{
                                                Ok(content)=>{
                                                    self.current_markdown_content=content;
                                                    self.right_panel_mode=RightPanelMode::Preview;
                                                }
                                                Err(content)=>{
                                                    eprintln!("error convert: {}", content);
                                                }
                                            }
                                        }
                                        else {
                                            eprintln!("invalid file path!{}",path.display());
                                        }
                                        
                                    }
                                    else{
                                       println!("Please select files!") // add new ui??
                                    }
                                    
                                }
                            }//end for 
                        }//end else
                        }); // end scroll area;

                    });//end vertical_centered
            });//end left side panel
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
                                let viewer = CommonMarkViewer::new("markdown_viewer_unique_id");
                                viewer.show(ui, &mut self.markdown_cache, &self.current_markdown_content);
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
                ui.add_space(10.0);
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
}

fn main(){
    createFrame();

}
