use infer;
mod config;
pub mod generator;
pub mod converter;

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
        "html" | "htm" => Some("text/html"),
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

    match mime_type {
        "audio/x-wav" | "audio/wav" | "audio/wave" => {
            generator::wav2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert WAV: {}", e))
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            generator::docx2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert DOCX: {}", e))
        }
        "image/jpeg" | "image/png" | "image/gif" => {
            generator::image2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert image: {}", e))
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
        "text/html" => {
            generator::html2md::run(&file.file_stream)
                .map_err(|e| format!("Failed to convert HTML: {}", e))
        }
        _ => Err(format!("Unsupported file type: {}", mime_type)),
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