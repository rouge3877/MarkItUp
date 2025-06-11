use std::process::{Command, Stdio};
use std::io::{Read, Write, Cursor};


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
    let mut child = Command::new("ffmpeg")
        .arg("-i")
        .arg("pipe:0")
        .arg("-vn")
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("-f")
        .arg("wav")
        .arg("pipe:1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法启动 ffmpeg 进程: {}", e))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "无法打开 ffmpeg stdin".to_string())?;
        stdin
            .write_all(input_bytes)
            .map_err(|e| format!("向 ffmpeg stdin 写入数据失败: {}", e))?;
    }

    let mut output = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout
            .read_to_end(&mut output)
            .map_err(|e| format!("读取 ffmpeg 输出流失败: {}", e))?;
    }

    let status = child.wait().map_err(|e| format!("等待 ffmpeg 进程退出时出错: {}", e))?;
    if status.success() {
        Ok(output)
    } else {
        let mut err_buf = Vec::new();
        if let Some(mut stderr) = child.stderr {
            let _ = stderr.read_to_end(&mut err_buf);
        }
        let err_msg = String::from_utf8_lossy(&err_buf);
        Err(format!("ffmpeg 转换失败: {}", err_msg))
    }
}

fn detect_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 12 {
        return None;
    }
    
    // WAV format
    if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return Some("wav");
    }
    
    // MP4/M4A format
    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        return Some("mp4");
    }
    
    // WebM format
    if bytes.len() >= 4 && &bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return Some("webm");
    }
    
    // MP3 format
    if bytes.len() >= 3 && (&bytes[0..3] == b"ID3" || &bytes[0..2] == [0xFF, 0xFB] || &bytes[0..2] == [0xFF, 0xF3] || &bytes[0..2] == [0xFF, 0xF2]) {
        return Some("mp3");
    }
    
    // OGG format
    if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
        return Some("ogg");
    }
    
    // FLAC format
    if bytes.len() >= 4 && &bytes[0..4] == b"fLaC" {
        return Some("flac");
    }
    
    None
}

fn convert_to_mono_wav(audio_data: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    let mut wav_data = Vec::new();
    let mut cursor = Cursor::new(&mut wav_data);
    
    // Convert to mono if stereo
    let mono_data: Vec<f32> = if channels == 1 {
        audio_data.to_vec()
    } else {
        audio_data.chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    
    // Convert f32 to i16 PCM
    let pcm_data: Vec<i16> = mono_data.iter()
        .map(|&sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect();
    
    // Write WAV header
    let data_size = pcm_data.len() * 2; // 2 bytes per i16 sample
    let file_size = 36 + data_size;
    
    // RIFF header
    cursor.write_all(b"RIFF").map_err(|e| format!("写入RIFF头失败: {}", e))?;
    cursor.write_all(&(file_size as u32).to_le_bytes()).map_err(|e| format!("写入文件大小失败: {}", e))?;
    cursor.write_all(b"WAVE").map_err(|e| format!("写入WAVE标识失败: {}", e))?;
    
    // Format chunk
    cursor.write_all(b"fmt ").map_err(|e| format!("写入fmt chunk失败: {}", e))?;
    cursor.write_all(&16u32.to_le_bytes()).map_err(|e| format!("写入chunk大小失败: {}", e))?; // PCM format chunk size
    cursor.write_all(&1u16.to_le_bytes()).map_err(|e| format!("写入音频格式失败: {}", e))?; // PCM format
    cursor.write_all(&1u16.to_le_bytes()).map_err(|e| format!("写入声道数失败: {}", e))?; // Mono
    cursor.write_all(&sample_rate.to_le_bytes()).map_err(|e| format!("写入采样率失败: {}", e))?;
    cursor.write_all(&(sample_rate * 2).to_le_bytes()).map_err(|e| format!("写入字节率失败: {}", e))?; // Byte rate
    cursor.write_all(&2u16.to_le_bytes()).map_err(|e| format!("写入块对齐失败: {}", e))?; // Block align
    cursor.write_all(&16u16.to_le_bytes()).map_err(|e| format!("写入位深失败: {}", e))?; // Bits per sample
    
    // Data chunk
    cursor.write_all(b"data").map_err(|e| format!("写入data chunk失败: {}", e))?;
    cursor.write_all(&(data_size as u32).to_le_bytes()).map_err(|e| format!("写入数据大小失败: {}", e))?;
    
    // Write PCM data
    for &sample in &pcm_data {
        cursor.write_all(&sample.to_le_bytes()).map_err(|e| format!("写入PCM数据失败: {}", e))?;
    }
    
    Ok(wav_data)
}

fn extract_audio_simple(bytes: &[u8], format: &str) -> Result<(Vec<f32>, u32, u16), String> {
    match format {
        "wav" => {
            // Simple WAV parsing - this is a basic implementation
            if bytes.len() < 44 {
                return Err("WAV文件太小".into());
            }
            
            // Read format info from WAV header
            let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
            let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
            let bits_per_sample = u16::from_le_bytes([bytes[34], bytes[35]]);
            
            if bits_per_sample != 16 {
                return Err("目前只支持16位WAV文件".into());
            }
            
            // Find data chunk
            let mut pos = 36;
            while pos + 8 < bytes.len() {
                if &bytes[pos..pos+4] == b"data" {
                    let data_size = u32::from_le_bytes([bytes[pos+4], bytes[pos+5], bytes[pos+6], bytes[pos+7]]) as usize;
                    let data_start = pos + 8;
                    let data_end = data_start + data_size;
                    
                    if data_end > bytes.len() {
                        return Err("WAV数据块大小超出文件范围".into());
                    }
                    
                    // Convert i16 PCM to f32
                    let pcm_data = &bytes[data_start..data_end];
                    let mut audio_data = Vec::new();
                    
                    for chunk in pcm_data.chunks(2) {
                        if chunk.len() == 2 {
                            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                            audio_data.push(sample as f32 / 32768.0);
                        }
                    }
                    
                    return Ok((audio_data, sample_rate, channels));
                }
                pos += 1;
            }
            
            Err("未找到WAV数据块".into())
        },
        _ => {
            // For other formats, we would need more sophisticated decoding
            // This is a placeholder that indicates the format is not supported in manual mode
            Err(format!("手动模式不支持{}格式，需要ffmpeg进行转换", format))
        }
    }
}

fn run_manual_fallback(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let format = detect_format(input_bytes)
        .ok_or_else(|| "无法识别输入文件格式".to_string())?;
    
    let (audio_data, sample_rate, channels) = extract_audio_simple(input_bytes, format)?;
    
    convert_to_mono_wav(&audio_data, sample_rate, channels)
}

pub fn video_to_wav(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if is_ffmpeg_available() {
        run_with_ffmpeg(input_bytes)
    } else {
        run_manual_fallback(input_bytes)
    }
}