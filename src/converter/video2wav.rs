use std::process::{Command, Stdio};
use std::io::{Read};
use std::fs;
use std::path::PathBuf;

fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_with_ffmpeg(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // Create temporary files in tmpfs (/tmp)
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("markitup_input_{}", std::process::id()));
    let output_path = temp_dir.join(format!("markitup_output_{}.wav", std::process::id()));
    
    // Write input data to temporary file
    fs::write(&input_path, input_bytes)
        .map_err(|e| format!("写入临时输入文件失败: {}", e))?;
    
    // Ensure cleanup on function exit
    let _cleanup = TempFileCleanup {
        paths: vec![input_path.clone(), output_path.clone()],
    };
    
    // Run ffmpeg with file input/output
    let mut child = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_path)
        .arg("-vn")                    // No video
        .arg("-ac")
        .arg("1")                      // Mono output
        .arg("-ar")
        .arg("16000")                  // 16kHz sample rate for speech
        .arg("-acodec")
        .arg("pcm_s16le")              // 16-bit PCM
        .arg("-f")
        .arg("wav")
        .arg("-y")                     // Overwrite output
        .arg(&output_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法启动 ffmpeg 进程: {}", e))?;

    // Read stderr for error reporting
    let mut stderr_handle = child.stderr.take();
    let stderr_thread = std::thread::spawn(move || {
        let mut err_buf = Vec::new();
        if let Some(ref mut stderr) = stderr_handle {
            let _ = stderr.read_to_end(&mut err_buf);
        }
        err_buf
    });

    // Read stdout (should be minimal since we're writing to file)
    let mut stdout_buf = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_end(&mut stdout_buf);
    }

    let status = child.wait().map_err(|e| format!("等待 ffmpeg 进程退出时出错: {}", e))?;
    let err_buf = stderr_thread.join().unwrap_or_default();
    
    if status.success() {
        // Check if output file exists and has content
        match fs::metadata(&output_path) {
            Ok(metadata) if metadata.len() > 0 => {
                // Read the generated WAV file
                fs::read(&output_path)
                    .map_err(|e| format!("读取输出WAV文件失败: {}", e))
            },
            Ok(_) => {
                let err_msg = String::from_utf8_lossy(&err_buf);
                Err(format!("ffmpeg 生成了空文件。错误信息: {}", err_msg))
            },
            Err(e) => {
                let err_msg = String::from_utf8_lossy(&err_buf);
                Err(format!("输出文件不存在: {}。ffmpeg错误: {}", e, err_msg))
            }
        }
    } else {
        let err_msg = String::from_utf8_lossy(&err_buf);
        Err(format!("ffmpeg 转换失败 (退出码: {:?}): {}", status.code(), err_msg))
    }
}

// Helper struct for automatic cleanup of temporary files
struct TempFileCleanup {
    paths: Vec<PathBuf>,
}

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = fs::remove_file(path);
        }
    }
}

fn detect_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 12 {
        return None;
    }
    
    // WAV format - RIFF header
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return Some("wav");
    }
    
    // MP4/M4A/MOV format - ftyp box
    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        return Some("mp4");
    }
    
    // WebM format - EBML header
    if bytes.len() >= 4 && &bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return Some("webm");
    }
    
    // MP3 format - ID3 tag or frame sync
    if bytes.len() >= 3 {
        if &bytes[0..3] == b"ID3" {
            return Some("mp3");
        }
        if bytes.len() >= 2 {
            let header = u16::from_be_bytes([bytes[0], bytes[1]]);
            // MP3 frame sync patterns
            if (header & 0xFFE0) == 0xFFE0 {
                return Some("mp3");
            }
        }
    }
    
    // OGG format
    if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
        return Some("ogg");
    }
    
    // FLAC format
    if bytes.len() >= 4 && &bytes[0..4] == b"fLaC" {
        return Some("flac");
    }

    // AVI format 
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"AVI " {
        return Some("avi");
    }

    // QuickTime/MOV format
    if bytes.len() >= 8 {
        let size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if size >= 8 && size < bytes.len() && &bytes[4..8] == b"moov" {
            return Some("mov");
        }
    }
    
    None
}

fn create_wav_header(data_size: usize, sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    let file_size = (36 + data_size) as u32;
    let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = channels * (bits_per_sample / 8);
    
    // RIFF header
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    
    // Format chunk
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // PCM format chunk size
    header.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    header.extend_from_slice(&channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&block_align.to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    
    // Data chunk header
    header.extend_from_slice(b"data");
    header.extend_from_slice(&(data_size as u32).to_le_bytes());
    
    header
}

fn convert_to_standard_wav(audio_data: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    // Target: 16kHz mono 16-bit PCM for speech recognition
    let target_sample_rate = 16000u32;
    let target_channels = 1u16;
    
    // Convert to mono first
    let mono_data: Vec<f32> = if channels == 1 {
        audio_data.to_vec()
    } else {
        audio_data.chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    
    // Resample if needed (simple linear interpolation)
    let resampled_data: Vec<f32> = if sample_rate != target_sample_rate {
        let ratio = sample_rate as f64 / target_sample_rate as f64;
        let output_len = (mono_data.len() as f64 / ratio) as usize;
        
        (0..output_len)
            .map(|i| {
                let src_idx = (i as f64 * ratio) as usize;
                if src_idx < mono_data.len() {
                    mono_data[src_idx]
                } else {
                    0.0
                }
            })
            .collect()
    } else {
        mono_data
    };
    
    // Convert f32 to i16 PCM with proper clipping
    let pcm_data: Vec<i16> = resampled_data.iter()
        .map(|&sample| {
            let clamped = sample.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect();
    
    // Create WAV file
    let data_size = pcm_data.len() * 2;
    let mut wav_data = create_wav_header(data_size, target_sample_rate, target_channels, 16);
    
    // Append PCM data
    for &sample in &pcm_data {
        wav_data.extend_from_slice(&sample.to_le_bytes());
    }
    
    Ok(wav_data)
}

fn parse_wav_file(bytes: &[u8]) -> Result<(Vec<f32>, u32, u16), String> {
    if bytes.len() < 44 {
        return Err("WAV文件头过短".into());
    }
    
    // Verify RIFF/WAVE header
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("无效的WAV文件格式".into());
    }
    
    // Find fmt chunk
    let mut pos = 12;
    let mut fmt_found = false;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits_per_sample = 0u16;
    
    while pos + 8 <= bytes.len() {
        let chunk_id = &bytes[pos..pos+4];
        let chunk_size = u32::from_le_bytes([
            bytes[pos+4], bytes[pos+5], bytes[pos+6], bytes[pos+7]
        ]) as usize;
        
        if chunk_id == b"fmt " {
            if chunk_size < 16 || pos + 8 + chunk_size > bytes.len() {
                return Err("fmt chunk 数据不完整".into());
            }
            
            let format_tag = u16::from_le_bytes([bytes[pos+8], bytes[pos+9]]);
            if format_tag != 1 {
                return Err("只支持PCM格式的WAV文件".into());
            }
            
            channels = u16::from_le_bytes([bytes[pos+10], bytes[pos+11]]);
            sample_rate = u32::from_le_bytes([
                bytes[pos+12], bytes[pos+13], bytes[pos+14], bytes[pos+15]
            ]);
            bits_per_sample = u16::from_le_bytes([bytes[pos+22], bytes[pos+23]]);
            
            if bits_per_sample != 16 && bits_per_sample != 8 && bits_per_sample != 24 && bits_per_sample != 32 {
                return Err(format!("不支持的位深: {} 位", bits_per_sample));
            }
            
            fmt_found = true;
        } else if chunk_id == b"data" && fmt_found {
            if pos + 8 + chunk_size > bytes.len() {
                return Err("data chunk 数据超出文件范围".into());
            }
            
            let audio_data = parse_pcm_data(&bytes[pos+8..pos+8+chunk_size], bits_per_sample)?;
            return Ok((audio_data, sample_rate, channels));
        }
        
        pos += 8 + chunk_size;
        // Align to word boundary
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    if !fmt_found {
        Err("未找到fmt chunk".into())
    } else {
        Err("未找到data chunk".into())
    }
}

fn parse_pcm_data(data: &[u8], bits_per_sample: u16) -> Result<Vec<f32>, String> {
    let mut audio_data = Vec::new();
    
    match bits_per_sample {
        8 => {
            for &byte in data {
                let sample = (byte as i16 - 128) as f32 / 128.0;
                audio_data.push(sample);
            }
        },
        16 => {
            for chunk in data.chunks(2) {
                if chunk.len() == 2 {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    audio_data.push(sample as f32 / 32768.0);
                }
            }
        },
        24 => {
            for chunk in data.chunks(3) {
                if chunk.len() == 3 {
                    let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]) >> 8;
                    audio_data.push(sample as f32 / 8388608.0);
                }
            }
        },
        32 => {
            for chunk in data.chunks(4) {
                if chunk.len() == 4 {
                    let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    audio_data.push(sample as f32 / 2147483648.0);
                }
            }
        },
        _ => return Err(format!("不支持的位深: {}", bits_per_sample)),
    }
    
    Ok(audio_data)
}

fn run_manual_fallback(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let format = detect_format(input_bytes)
        .ok_or_else(|| "无法识别输入文件格式".to_string())?;
    
    match format {
        "wav" => {
            let (audio_data, sample_rate, channels) = parse_wav_file(input_bytes)?;
            convert_to_standard_wav(&audio_data, sample_rate, channels)
        },
        _ => {
            Err(format!("手动模式不支持 {} 格式，需要 ffmpeg 进行转换", format))
        }
    }
}

pub fn video_to_wav(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if input_bytes.is_empty() {
        return Err("输入数据为空".to_string());
    }

    if is_ffmpeg_available() {
        run_with_ffmpeg(input_bytes)
    } else {
        run_manual_fallback(input_bytes)
    }
}