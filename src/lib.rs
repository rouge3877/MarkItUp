use infer;
pub mod config;
pub mod generator;
pub mod converter;
use config::{SETTINGS};

pub struct ConverterFile {
    pub file_path: Option<String>,
    pub file_stream: Vec<u8>,
}

// Helper function to determine file type from extension
fn get_file_type_from_extension(file_path: &Option<String>) -> Option<&'static str> {
    let path = file_path.as_ref()?;
    let extension = std::path::Path::new(path)
        .extension()?
        .to_str()?
        .to_lowercase();

    match extension.as_str() {
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        "csv" => Some("text/csv"),
        "wav" => Some("audio/wav"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "tiff" => Some("image/tiff"),
        "ico" => Some("image/x-icon"),
        "html" | "htm" => Some("text/html"),
        "pdf" => Some("application/pdf"),
        "mp3" => Some("audio/mpeg"),
        "flac" => Some("audio/flac"),
        "ogg" => Some("audio/ogg"),
        "aac" => Some("audio/aac"),
        "m4a" => Some("audio/x-m4a"),
        "mp4" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        "avi" => Some("video/avi"),
        "mpeg" => Some("video/mpeg"),
        _ => None,
    }
}

// byte_stream -> String
pub fn convert(file: ConverterFile) -> Result<String, String> {
    let kind = infer::get(&file.file_stream)
        .ok_or_else(|| "Could not determine file type".to_string())?;

    let mut mime_type = kind.mime_type();

    // Fallback to extension-based detection for ZIP files (Office documents) and text files
    if mime_type == "application/zip" || mime_type == "text/plain" {
        if let Some(extension_mime) = get_file_type_from_extension(&file.file_path) {
            mime_type = extension_mime;
        }
    }

    if cfg!(debug_assertions) {
        dbg!(mime_type);
    }

    let markdown = match mime_type {
        "audio/x-wav" | "audio/wav" | "audio/wave" | "audio/mpeg" | "audio/mp3" | "audio/flac" | "audio/ogg" | "audio/aac" | "audio/x-m4a" => {
            // Convert other audio formats to WAV first
            let wav_data = converter::audio2wav::audio_to_wav(&file.file_stream)
                .map_err(|e| format!("Failed to convert audio to WAV: {:?}", e))?;

            // printf information when debug
            if cfg!(debug_assertions) {
                dbg!(wav_data.len());
            }
            
            generator::wav2md::run(&wav_data)
                .map_err(|e| format!("Failed to convert WAV: {}", e))
        }
        // All kind of video formats, transform to wav
        "video/mp4" | "video/x-matroska" | "video/webm" | "video/avi" | "video/mpeg" => {
            if cfg!(debug_assertions) {
                dbg!(file.file_stream.len());
            }
            let wav_data = converter::video2wav::video_to_wav(&file.file_stream)
                .map_err(|e| format!("Failed to convert video to WAV: {}", e))?;
            if cfg!(debug_assertions) {
                dbg!(wav_data.len());
            }
            let _wav_data = converter::audio2wav::audio_to_wav(&wav_data)
                .map_err(|_| format!("Failed to convert video WAV to standard WAV"))?;
            if cfg!(debug_assertions) {
                dbg!(_wav_data.len());
            }
            generator::wav2md::run(&_wav_data)
                .map_err(|e| format!("Failed to convert WAV from video: {}", e))
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            generator::docx2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert DOCX: {}", e))
        }
        "image/jpeg" | "image/png" | "image/gif" => {
            generator::image2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert image: {}", e))
        }
        // more image formats can be added here
        "image/webp" | "image/bmp" | "image/tiff" | "image/ico" => {
            let png_data = converter::image2png::image_to_png(&file.file_stream)
                .map_err(|e| format!("Failed to convert image to PNG: {}", e))?;
            
            generator::image2md::run(&png_data)
                .map_err(|e| format!("Failed to convert PNG image: {}", e))
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            generator::pptx2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert PPTX: {}", e))
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            let csvs = converter::xlsx2csv::xlsx_to_csv(&file.file_stream, None)
                .map_err(|e| format!("Failed to convert XLSX: {}", e))?;
            
            let mut combined_md = String::new();
            
            for (name, csv) in csvs.sheet_names.iter().zip(csvs.csv_data.iter()) {
                if cfg!(debug_assertions) {
                    dbg!(name);
                }
                let md = generator::csv2md::run(csv.as_bytes())
                    .map_err(|e| format!("Failed to convert CSV for sheet '{}': {}", name, e))?;
                
                // Add sheet name as header and the markdown content
                if !combined_md.is_empty() {
                    combined_md.push_str("\n\n---\n\n");
                }
                combined_md.push_str(&format!("## Sheet: {}\n\n", name));
                combined_md.push_str(&md);
            }
            
            if combined_md.is_empty() {
                Err("No sheets found in XLSX file".to_string())
            } else {
                Ok(combined_md)
            }
        }
        "text/csv" | "application/csv" => {
            generator::csv2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert CSV: {}", e))
        }
        "application/pdf" => {
            generator::pdf2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert PDF: {}", e))
        }
        "text/html" => {
            generator::html2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert HTML: {}", e))
        }
        _ => Err(format!("Unsupported file type: {}", mime_type)),
    };

    let cfg = &*SETTINGS.read().unwrap();
    if cfg.is_ai_sweep {
        ai_sweep(markdown)
    } else {
        markdown
    }
}

pub fn convert_from_path(file_path: &str) -> Result<String, String> {
    let file_stream = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

    let file = ConverterFile {
        file_path: Some(file_path.to_string()),
        file_stream,
    };

    convert(file)
}

fn ai_sweep(markdown: Result<String, String>) -> Result<String, String> {
    let markdown_content = markdown?;
    
    // Check if the markdown contains base64 encoded images
    if contains_base64_images(&markdown_content) {
        if cfg!(debug_assertions) {
            eprintln!("Detected base64 images in markdown, skipping AI sweep");
        }
        return Ok(markdown_content);
    }
    
    let cfg = &*SETTINGS.read().unwrap();
    let api_key = cfg.deepseek_api_key.as_ref()
        .ok_or_else(|| "DeepSeek API key is not configured".to_string())?;
    
    format_markdown_with_deepseek(&markdown_content, api_key)
}

fn contains_base64_images(markdown: &str) -> bool {
    // Check for common base64 image patterns in markdown
    let base64_patterns = [
        "data:image/png;base64,",
        "data:image/jpg;base64,",
        "data:image/jpeg;base64,",
        "data:image/gif;base64,",
        "data:image/webp;base64,",
        "data:image/bmp;base64,",
        "data:image/svg+xml;base64,",
    ];
    
    for pattern in &base64_patterns {
        if markdown.contains(pattern) {
            return true;
        }
    }
    
    // Also check for markdown image syntax with base64 data URLs
    if markdown.contains("![") && markdown.contains("data:image/") && markdown.contains("base64,") {
        return true;
    }
    
    false
}

fn create_format_prompt(markdown: &str) -> String {
    format!(
        "Please format and fix the markdown below. Only fix formatting issues like spacing, alignment, and markdown syntax. Do not modify any content, structure, or meaning. Return ONLY the formatted markdown content without any explanations, comments, or additional text. Even \"```markdown\" and \"```\" are not allowed output:\n\n{}",
        markdown
    )
}

fn format_markdown_with_deepseek(markdown: &str, api_key: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let prompt = create_format_prompt(markdown);
    
    // 对 content 进行预处理，确保 JSON 安全
    let escaped_content = escape_json_string(&prompt);
    
    // 直接构造 JSON 字符串
    let json_payload = format!(
        r#"{{
            "model": "deepseek-chat",
            "messages": [
                {{
                    "role": "user",
                    "content": "{}"
                }}
            ],
            "max_tokens": 8192,
            "temperature": 0.1
        }}"#,
        escaped_content
    );

    if cfg!(debug_assertions) {
        eprintln!("Sending request to DeepSeek API...");
        eprintln!("Payload length: {} bytes", json_payload.len());
    }

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(json_payload)
        .send()
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let status = response.status();
    let response_text = response.text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if cfg!(debug_assertions) {
        eprintln!("Response status: {}", status);
        if response_text.len() < 2000 {
            eprintln!("Full response: {}", response_text);
        } else {
            eprintln!("Response preview (first 1000 chars): {}", &response_text[..1000]);
        }
    }

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, response_text));
    }

    // 提取 content
    extract_deepseek_content(&response_text)
}

fn escape_json_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\u{08}' => "\\b".to_string(),
            '\u{0C}' => "\\f".to_string(),
            c if c.is_control() => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}

fn extract_deepseek_content(response_text: &str) -> Result<String, String> {
    // 首先尝试解析为 JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_text) {
        // 标准 OpenAI 格式
        if let Some(choices) = json["choices"].as_array() {
            if let Some(first_choice) = choices.first() {
                if let Some(content) = first_choice["message"]["content"].as_str() {
                    return Ok(content.to_string());
                }
            }
        }
    }

    // 如果 JSON 解析失败，使用字符串匹配
    // 查找 "content":"..." 模式
    if let Some(start) = response_text.find(r#""content":"#) {
        let content_start = start + 11; // "content":" 的长度
        let remaining = &response_text[content_start..];
        
        // 找到内容的结束位置，需要正确处理转义字符
        if let Some(content_end) = find_json_string_end(remaining) {
            let raw_content = &remaining[..content_end];
            // 解码 JSON 转义字符
            let decoded = decode_json_string(raw_content);
            return Ok(decoded);
        }
    }

    Err(format!("Could not extract content from DeepSeek response. Response length: {} bytes", response_text.len()))
}

fn find_json_string_end(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        match chars[i] {
            '"' => return Some(i), // 找到结束引号
            '\\' => {
                // 跳过转义字符
                i += 2;
            }
            _ => i += 1,
        }
    }
    
    None
}

fn decode_json_string(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\u{08}'),
                'f' => result.push('\u{0C}'),
                'u' if i + 5 < chars.len() => {
                    // Unicode 转义 \uXXXX
                    let hex: String = chars[i+2..i+6].iter().collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(unicode_char) = std::char::from_u32(code) {
                            result.push(unicode_char);
                        }
                    }
                    i += 6;
                    continue;
                }
                c => {
                    result.push('\\');
                    result.push(c);
                }
            }
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    
    result
}
