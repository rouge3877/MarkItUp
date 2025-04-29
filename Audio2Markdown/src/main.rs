// src/main.rs
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use hound::WavReader;
use std::path::PathBuf;
use std::fs;
use vosk::{Model, Recognizer};

#[derive(Parser)]
#[command(name = "Audio2Markdown")]
#[command(about = "Convert audio to Markdown", version)]
struct Cli {
    input: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // 加载模型
    let model = Model::new("models/vosk-model-small-en-us-0.15")
        .ok_or(anyhow!("模型加载失败"))?;

    // 读取音频文件
    let (samples, sample_rate) = read_wave_file(&args.input)?;
    if sample_rate != 16000 {
        anyhow::bail!("需要 16000Hz 采样率，当前为 {}", sample_rate);
    }

    // 执行识别
    let mut recognizer = Recognizer::new(&model, 16000.0)
        .ok_or(anyhow!("识别器初始化失败"))?;
    recognizer.accept_waveform(&samples);
    let result =recognizer.final_result();
    let text = result
        .single()
        .map(|alt| alt.text)
        .unwrap_or_else(|| "[未识别到有效内容]");

    // 6. 生成增强版Markdown
    let markdown = format!(
        "# 音频转录\n\n\
        ## 基本信息\n\
        - **原始文件**: `{}`\n\
        - **采样率**: {} Hz\n\
        - **识别引擎**: Vosk 0.3.1\n\n\
        ## 转录内容\n{}",
        args.input.display(),
        sample_rate,
        text
    );
    fs::write(&args.output, markdown)?;

    println!("转换成功！输出文件: {}", args.output.display());
    Ok(())
}

// read_wave_file 函数保持不变
fn read_wave_file(path: &PathBuf) -> Result<(Vec<i16>, u32)> {
    let reader = WavReader::open(path)
        .with_context(|| format!("无法打开音频文件: {}", path.display()))?;

    let spec = reader.spec();
    if spec.channels != 1 {
        anyhow::bail!("需要单声道音频");
    }
    if spec.bits_per_sample != 16 {
        anyhow::bail!("需要 16-bit 位深");
    }

    let samples: Vec<i16> = reader.into_samples()
        .collect::<Result<_, _>>()?;

    Ok((samples, spec.sample_rate))
}
