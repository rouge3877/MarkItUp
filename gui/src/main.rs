use eframe::{egui, run_native, NativeOptions,CreationContext};
use egui::{ CentralPanel, Button, Sense, ScrollArea,ViewportBuilder,Context,FontDefinitions,FontFamily,TextStyle, FontId,};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
pub static FILE_LIST: once_cell::sync::Lazy<Arc<Mutex<Vec<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

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
        style.text_styles.insert(TextStyle::Button, FontId::proportional(16.0));
        style.text_styles.insert(TextStyle::Body, FontId::proportional(14.0));
        style.text_styles.insert(TextStyle::Heading, FontId::proportional(20.0));
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
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui|{
                ui.add_space(50.0);
                let FileButton = Button::new("Add file").sense(Sense::click());
                let FileButtonResponse = ui.add_sized([150.0, 40.0], FileButton);

                if FileButtonResponse.clicked() {
                    // 按钮被点击了！
                    load_file();
                    self.button1_clicks += 1;
                    println!("按钮一被点击了！ 点击次数：{}", self.button1_clicks);
                }
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
                let ConvertButton = Button::new("Convert").sense(Sense::click());
                let ConvertButtonResponse = ui.add_sized([150.0, 40.0], ConvertButton);

                if ConvertButtonResponse.clicked() {
                    // 按钮被点击了！
                    self.button2_clicks += 1;
                    println!("按钮2被点击了！ 点击次数：{}", self.button2_clicks);
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
            .with_fullscreen(true) // <-- 这里是设置全屏的关键
            // .with_inner_size(egui::vec2(800.0, 600.0)) // 如果需要，也可以设置初始大小，但全屏时可能不相关
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

