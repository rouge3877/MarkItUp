use crate::config::SETTINGS;
use base64::Engine;
use std::fs;

pub enum ImageProcessingMode {
    Base64,
    SaveToFile,
}


pub fn run(file_stream: &[u8]) -> Result<String, String> {
    let cfg = &*SETTINGS.read().unwrap();
    
    // Determine mode based on global config: if image_path is empty, use base64
    let mode = if cfg.image_path.as_os_str().is_empty() {
        ImageProcessingMode::Base64
    } else {
        ImageProcessingMode::SaveToFile
    };
    
    run_with_mode(file_stream, mode)
}


pub fn run_with_mode(file_stream: &[u8], mode: ImageProcessingMode) -> Result<String, String> {
    let cfg = &*SETTINGS.read().unwrap();

    if file_stream.is_empty() {
        return Err("Input stream is empty".to_string());
    }

    // Determine the MIME type and extension of the image
    let (mime_type, extension) = if let Some(kind) = infer::get(file_stream) {
        let mime = kind.mime_type().to_string();
        let ext = match kind.mime_type() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg", // default fallback
        };
        (mime, ext)
    } else {
        ("image/jpeg".to_string(), "jpg")
    };

    let image_name = if cfg.is_ai_enpower {
        ai_generate_name_from_bytes(file_stream, &mime_type)
    } else {
        // generate a timestamp-based name
        let timestamp = chrono::Utc::now().timestamp();
        format!("pic-{}", timestamp)
    };

    match mode {
        ImageProcessingMode::Base64 => {
            // Encode the image data to base64
            let encoded = base64::engine::general_purpose::STANDARD.encode(file_stream);
            let md_content = format!("![{}](data:{};base64,{})", image_name, mime_type, encoded);
            Ok(md_content)
        }
        ImageProcessingMode::SaveToFile => {
            // Save image to file and return markdown reference
            let filename = format!("{}.{}", image_name, extension);
            let file_path = cfg.image_path.join(&filename);
            
            // Ensure the directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create image directory: {}", e))?;
            }
            
            // Write the image file
            fs::write(&file_path, file_stream)
                .map_err(|e| format!("Failed to save image file: {}", e))?;
            
            // Return markdown reference to the saved file (just the filename for relative path)
            let md_content = format!("![{}]({})", image_name, filename);
            Ok(md_content)
        }
    }
}


fn ai_generate_name_from_bytes(file_stream: &[u8], mime_type: &str) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(file_stream);
    ai_generate_name(encoded, mime_type)
}


fn ai_generate_name(encoded: String, mime_type: &str) -> String {
    // Try to generate name using Doubao API, fallback to timestamp if failed
    match call_doubao_api(&encoded, mime_type) {
        Ok(name) => name,
        Err(_) => {
            // Fallback to timestamp-based name if AI call fails
            let timestamp = chrono::Utc::now().timestamp();
            format!("pic-{}", timestamp)
        }
    }
}

fn call_doubao_api(encoded_image: &str, mime_type: &str) -> Result<String, Box<dyn std::error::Error>> {
    use serde_json::json;
    
    // Doubao API endpoint and key (you should configure these in your SETTINGS)
    let api_url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
    let cfg = &*SETTINGS.read().unwrap();
    let api_key = cfg.doubao_api_key.as_ref()
        .ok_or("Doubao API key not configured")?;
    
    // Prepare the request payload using serde_json::json! macro
    let payload = json!({
        "model": "doubao-1-5-thinking-vision-pro-250428",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "Please analyze this image and generate a short, descriptive filename (without extension) in English. The name should be concise and describe the main subject or content of the image. Only return the filename, nothing else."
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{};base64,{}", mime_type, encoded_image)
                        }
                    }
                ]
            }
        ],
        "max_tokens": 50,
        "temperature": 0.7
    });
    
    // Make HTTP request
    let client = ureq::Agent::new();
    let response = client
        .post(api_url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .send_json(&payload)?;

    // print response status and headers for debugging
    if response.status() != 200 {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    // Parse response
    let response_json: serde_json::Value = response.into_json()?;
    
    let generated_name = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("generated-image")
        .trim()
        .replace(" ", "-")
        .replace("/", "-")
        .replace("\\", "-")
        .replace(":", "-")
        .replace("*", "-")
        .replace("?", "-")
        .replace("\"", "-")
        .replace("<", "-")
        .replace(">", "-")
        .replace("|", "-");
    
    Ok(generated_name)
}
