use quick_xml::events::Event;
use quick_xml::Reader;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};

mod model;
use model::XmlToMarkdown::XmlToMarkdownConverter;

fn main() {
    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    // 读取输入内容
    let input = if args.len() >= 2 {
        // 从文件读取
        let input_path = &args[1];
        std::fs::read_to_string(input_path).expect("Failed to read input file")
    } else {
        // 从标准输入读取
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };

    // 转换 XML 到 Markdown
    let markdown = XmlToMarkdownConverter::convert_string(&input).expect("Conversion failed");

    // 处理输出
    if args.len() >= 3 {
        // 输出到文件
        let output_file = File::create(&args[2]).expect("Failed to create output file");
        let mut writer = BufWriter::new(output_file);
        writer
            .write_all(markdown.as_bytes())
            .expect("Failed to write to file");
    } else {
        // 输出到标准输出
        println!("{}", markdown);
    }
}
