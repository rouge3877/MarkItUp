use clap::{Parser, Subcommand};
use base64::Engine;
use std::path::{Path, PathBuf};
use std::fs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

// 子命令枚举
#[derive(Subcommand)]
enum Command {
    Convert{
        #[arg(short,long)]
        input:PathBuf,
        #[arg(short,long)]
        output:PathBuf,
    },
    Mv{
        #[arg(short,long)]
        input:PathBuf,
        #[arg(short,long)]
        output:PathBuf,
    }
    
}
fn convert_to_Base64(input:&Path,output:&Path) ->Result<()>{
    let args = Args::parse();
    let img_data = std::fs::read(input)
        .with_context(|| format!("无法读取输入文件: {}",input.display()))?;

    // Base64 编码
    let encoded = base64::engine::general_purpose::STANDARD.encode(&img_data);

    // 获取 MIME 类型
    let mime_type = mime_guess::from_path(input)
        .first_or_octet_stream()
        .to_string();

    // 构建 Markdown 内容
    let md_content = format!("![](data:{};base64,{})", mime_type, encoded);

    // 写入输出文件
    std::fs::write(output, &md_content)
        .with_context(|| format!("无法写入输出文件: {}", output.display()))?;

    println!(
        "转换成功！已将 {} 转换为 {}",
        input.display(),
        output.display()
    );
    Ok(())

}
fn mv(input:&Path,output:&Path) -> Result<()>{
    fs::rename(input,output).or_else(|_|{
        fs::copy(input,output)?;
        fs::remove_file(input)?;
        Ok(())
    })
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command{
        Command::Convert{input,output}=>{
            convert_to_Base64(&input,&output)?;
            println!("convert compeleted!");
        }
        Command::Mv{input,output}=>{
            mv(&input,&output)?;
            println!("move compeleted");
        }
    }
    Ok(())
}
