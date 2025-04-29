use anyhow::Context;
use base64::Engine;
use clap::Parser;
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 输入图片路径
    #[arg(short, long)]
    input: PathBuf,

    /// 输出 Markdown 文件路径
    #[arg(short, long)]
    output: PathBuf,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 读取图片文件
    let img_data = std::fs::read(&args.input)
        .with_context(|| format!("无法读取输入文件: {}", args.input.display()))?;

    // Base64 编码
    let encoded = base64::engine::general_purpose::STANDARD.encode(&img_data);

    // 获取 MIME 类型
    let mime_type = mime_guess::from_path(&args.input)
        .first_or_octet_stream()
        .to_string();

    // 构建 Markdown 内容
    let md_content = format!("![](data:{};base64,{})", mime_type, encoded);

    // 写入输出文件
    std::fs::write(&args.output, &md_content)
        .with_context(|| format!("无法写入输出文件: {}", args.output.display()))?;

    println!(
        "转换成功！已将 {} 转换为 {}",
        args.input.display(),
        args.output.display()
    );
    Ok(())
}
