use eframe::{egui, run_native, NativeOptions,CreationContext};
use egui::{Layout,TopBottomPanel, CentralPanel, Button, Sense, ScrollArea,ViewportBuilder,Context,FontDefinitions,FontFamily,TextStyle, FontId,SidePanel};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use once_cell::sync::Lazy;
use pulldown_cmark::{Parser,Options};
pub static FILE_LIST: once_cell::sync::Lazy<Arc<Mutex<Vec<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new()))); //file list is those files users add themselves by clicking button1
pub static DISPLAY_MD_LIST: Lazy<Arc<Mutex<Vec<PathBuf>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new()))); //this list contained markdown files which create by function in button 2
pub static SELECTED_MD_CONTENT: Lazy<Arc<Mutex<Option<String>>>> = // <-- THIS MUST BE HERE ,record which one file was chosen
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));
pub fn load_file()
{
    let result = FileDialog::new()
        .set_title("选择文件")
        .add_filter("所有文件", &["*"])
        .pick_files();
    match result{
        Some(paths)=>{
            let mut file_list = FILE_LIST.lock().unwrap();
            for path_buf in paths {
                if let Some(path_str) = path_buf.to_str() {
                    file_list.push(path_str.to_string());
                    println!("已添加文件: {}", path_str);
                } else {
                    eprintln!("无法将路径转换为 UTF-8 字符串: {:?}", path_buf);
                }
            }
        }
        None => {
            println!("file select canceled")
        }
    }
}
pub fn convert()
{
    let file_list_guard = FILE_LIST.lock().unwrap(); // 锁定 FILE_LIST (Vec<String>)
    let mut display_list_guard = DISPLAY_MD_LIST.lock().unwrap(); // 锁定 DISPLAY_MD_LIST (Vec<PathBuf>)

    display_list_guard.clear(); // 清空当前显示列表

    for path_str in file_list_guard.iter() {
        // 从 String 转换为 PathBuf
        let path_buf = PathBuf::from(path_str);
        display_list_guard.push(path_buf); // 将 PathBuf 存入 DISPLAY_MD_LIST
    }
    println!("DISPLAY_MD_LIST 已更新，当前有 {} 个文件。", display_list_guard.len());
}
fn render_raw_markdown_content(ui: &mut egui::Ui, markdown_text: &str) {
    // 直接使用 ui.label() 显示整个字符串
    // 使用 monospace 字体让代码看起来更规整，并设置一个合适的字体大小
    ui.label(egui::RichText::new(markdown_text).monospace().size(14.0));
}
struct MyApp{
    button1_clicks:u32, 
    button2_clicks:u32
}
impl Default for MyApp{
    fn default()->Self{
        Self{
            button1_clicks:0,
            button2_clicks:0
        }
    }
}
impl MyApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        // 1. Font Loading: This needs to happen here!
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            // Ensure font file path is correct, e.g., in a "fonts" folder at your project root
            egui::FontData::from_static(include_bytes!("../font.ttf"))
        );
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "chinese_font".to_owned());
        fonts.families.get_mut(&FontFamily::Monospace).unwrap()
            .insert(0, "chinese_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // 2. Style Setting: This also needs to happen here!
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(TextStyle::Button, FontId::proportional(22.0));
        style.text_styles.insert(TextStyle::Body, FontId::proportional(18.0));
        style.text_styles.insert(TextStyle::Heading, FontId::proportional(25.0));
        // You can set other TextStyle options as well
        cc.egui_ctx.set_style(style);

        // Return your MyApp instance
        Self { button1_clicks:0,
            button2_clicks:1,
        }
    }
}
impl eframe::App for MyApp{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        SidePanel::left("leftpenal").show(ctx, |ui| {
            TopBottomPanel::top("left_top_panel").show_inside(ui, |ui| {
                ui.vertical_centered(|ui|{
                    ui.add_space(50.0);
                    ScrollArea::vertical().show(ui, |ui| {
                        let file_list_guard = FILE_LIST.lock().unwrap();
                        if file_list_guard.is_empty() {
                            ui.label("当前没有文件。");
                        } 
                        else {
                            // 遍历并显示每个文件路径
                            for file_path in file_list_guard.iter() {
                                ui.label(file_path);
                            }
                        }
                    });
                    ui.add_space(50.0);
                    let FileButton = Button::new("Add file").sense(Sense::click());
                    let FileButtonResponse = ui.add_sized([150.0, 40.0], FileButton);

                    if FileButtonResponse.clicked() {
                        // 按钮被点击了！
                        load_file();
                        self.button1_clicks += 1;
                        println!("按钮一被点击了！ 点击次数：{}", self.button1_clicks);
                    }
                    ui.add_space(50.0);
                    let ConvertButton = Button::new("Convert").sense(Sense::click());
                    let ConvertButtonResponse = ui.add_sized([150.0, 40.0], ConvertButton);

                    if ConvertButtonResponse.clicked() {
                        // 按钮被点击了！
                        convert();
                        self.button2_clicks += 1;
                        println!("按钮2被点击了！ 点击次数：{}", self.button2_clicks);
                    }      
                });
            });
            ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                ui.heading("Markdown 文件列表");
                ui.add_space(5.0);

                // 使用 ScrollArea 来显示文件列表
                ScrollArea::vertical().show(ui, |ui| {
                    let file_list_guard = DISPLAY_MD_LIST.lock().unwrap();
                    if file_list_guard.is_empty() {
                        ui.label("当前没有 Markdown 文件。");
                    } else {
                        // 遍历文件列表并为每个文件创建按钮
                        for path_buf in file_list_guard.iter() {
                            if let Some(file_name) = path_buf.file_name().and_then(|n| n.to_str()) {
                                if ui.button(file_name).clicked() {
                                    // 用户点击文件，读取内容并更新全局状态
                                    match std::fs::read_to_string(path_buf) {
                                        Ok(content) => {
                                            *SELECTED_MD_CONTENT.lock().unwrap() = Some(content);
                                            println!("已加载文件内容: {:?}", path_buf.file_name());
                                        },
                                        Err(e) => {
                                            eprintln!("读取文件失败 {:?}: {}", path_buf, e);
                                            *SELECTED_MD_CONTENT.lock().unwrap() = Some(format!("Error loading file: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Markdown 内容预览");
            ui.add_space(10.0);

            ScrollArea::vertical().show(ui, |ui| {
                let md_content_guard = SELECTED_MD_CONTENT.lock().unwrap();
                if let Some(content) = md_content_guard.as_ref() {
                    // 调用我们新创建的渲染函数
                    render_raw_markdown_content(ui, content);
                } else {
                    ui.label("请在左侧选择一个 Markdown 文件进行预览。");
                }
            });
        });
    }
}
fn main() {
    let options = NativeOptions::default();
    let native_options = NativeOptions {
        // 使用 ViewportBuilder 来设置窗口属性，包括全屏
        viewport: ViewportBuilder::default()
           // .with_fullscreen(true) // <-- 这里是设置全屏的关键
             .with_inner_size(egui::vec2(1500.0, 1200.0)) // 如果需要，也可以设置初始大小，但全屏时可能不相关
            .with_maximized(true) // 可以同时设置最大化，这在某些平台效果会更好
            .with_title("文件选择示例"), // 也可以在这里设置标题
        // ... 其他选项，例如 VSync，多重采样等
        vsync: true, // 垂直同步，避免画面撕裂
        multisampling: 4, // 抗锯齿
        // ... 其他你可能需要的选项 ...
        ..Default::default()
    };
    eframe::run_native(
        "我的双按钮 GUI", // Window title
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    ).expect("Failed to run native app");
}

