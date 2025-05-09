use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use html2md::parse_html;
use std::fs;

#[derive(Parser)]
struct Cli {
    #[arg(short,long)]
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

fn read_html(path: &str) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    
    // 转换 PathBuf 到字符串引用
    let input_path = args.input.to_str()
        .ok_or("Invalid input path encoding")?;
    
    // 读取并转换
    let html = read_html(input_path)?;
    let md = parse_html(&html);
    
    // 写入文件
    fs::write(&args.output, md)?;
    
    println!("Converted successfully, file saved at {}", args.output.display());
    Ok(())
}
