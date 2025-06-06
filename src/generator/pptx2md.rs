use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Cursor, Read};
use zip::ZipArchive;
use crate::generator::image2md::{self, ImageProcessingMode};
use crate::config::SETTINGS;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
struct TableData {
    rows: Vec<Vec<String>>,
}

pub fn run(file_stream: &[u8]) -> Result<String, String> {
    run_with_images(file_stream)
}

fn run_with_images(file_stream: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(file_stream);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open PPTX archive: {}", e))?;

    // First, extract all images from the archive
    let mut images = HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access file in ZIP archive: {}", e))?;
        
        if file.name().starts_with("ppt/media/") {
            let mut image_data = Vec::new();
            file.read_to_end(&mut image_data)
                .map_err(|e| format!("Failed to read image data: {}", e))?;
            
            let filename = file.name().to_string();
            images.insert(filename, image_data);
        }
    }

    // Reset archive for slide processing
    let cursor = Cursor::new(file_stream);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open PPTX archive: {}", e))?;

    let mut markdown = String::new();
    markdown.push_str("# PowerPoint Presentation\n\n");

    let mut slide_num = 1;

    // Process all slides in the archive
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access file in ZIP archive: {}", e))?;
        
        if file.name().starts_with("ppt/slides/") && file.name().ends_with(".xml") {
            markdown.push_str(&format!("## Slide {}\n\n", slide_num));
            slide_num += 1;
            
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read slide content: {}", e))?;

            let slide_markdown = parse_slide_content(&content, &images)?;
            markdown.push_str(&slide_markdown);
            markdown.push_str("\n\n---\n\n");
        }
    }

    Ok(markdown)
}

fn parse_slide_content(
    xml_content: &str, 
    images: &HashMap<String, Vec<u8>>
) -> Result<String, String> {
    let mut reader = Reader::from_str(xml_content);
    let mut markdown = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(element)) => {
                match element.name().as_ref() {
                    b"p:txBody" => {
                        let text_content = extract_text_body(&mut reader)?;
                        if !text_content.trim().is_empty() {
                            markdown.push_str(&text_content);
                            markdown.push_str("\n\n");
                        }
                    }
                    b"a:tbl" => {
                        let table_content = extract_table(&mut reader)?;
                        markdown.push_str(&table_content);
                        markdown.push_str("\n");
                    }
                    b"a:blip" => {
                        if let Some(image_md) = process_image_element(&element, images)? {
                            markdown.push_str(&image_md);
                            markdown.push_str("\n\n");
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing slide XML: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(markdown)
}

fn process_image_element(
    element: &quick_xml::events::BytesStart,
    images: &HashMap<String, Vec<u8>>
) -> Result<Option<String>, String> {
    let cfg = &*SETTINGS.read().unwrap();
    
    // Determine processing mode based on configuration
    let mode = if cfg.image_path.as_os_str().is_empty() {
        ImageProcessingMode::Base64
    } else {
        ImageProcessingMode::SaveToFile
    };
    
    // Extract r:embed attribute to find the image
    for attr_result in element.attributes() {
        let attr = attr_result.map_err(|e| format!("Error reading attribute: {}", e))?;
        if attr.key.as_ref() == b"r:embed" {
            let embed_id = String::from_utf8_lossy(&attr.value);
            
            // Try to find matching image by filename patterns
            for (filename, image_data) in images {
                // Look for images that might match this embed ID or just process all images
                if filename.contains(&*embed_id) || 
                   filename.ends_with(".png") || 
                   filename.ends_with(".jpg") || 
                   filename.ends_with(".jpeg") ||
                   filename.ends_with(".gif") ||
                   filename.ends_with(".webp") {
                    
                    // Use the image2md module to process the image with proper mode
                    let image_md = image2md::run_with_mode(image_data, mode)?;
                    
                    // Handle relative paths if needed
                    let final_md = if !cfg.image_path.as_os_str().is_empty() {
                        adjust_image_path_in_markdown(image_md)?
                    } else {
                        image_md
                    };
                    
                    return Ok(Some(final_md));
                }
            }
            
            // If no matching image found, return a placeholder
            return Ok(Some(format!("![Image not found]({})", embed_id)));
        }
    }
    Ok(None)
}

fn adjust_image_path_in_markdown(markdown: String) -> Result<String, String> {
    let cfg = &*SETTINGS.read().unwrap();
    
    // If we have an output path and it's not empty, try to make image paths relative
    if let Some(output_path) = &cfg.output_path {
        if !output_path.as_os_str().is_empty() {
            // Calculate relative path from output directory to image directory
            let output_dir = output_path.parent().unwrap_or(Path::new("."));
            
            if let Ok(relative_path) = cfg.image_path.strip_prefix(output_dir) {
                let relative_str = relative_path.to_string_lossy();
                
                // Simple approach: since we control image generation, we can do direct replacement
                // Look for patterns like ![alt](filename.ext) and replace with ![alt](relative_path/filename.ext)
                let lines: Vec<&str> = markdown.lines().collect();
                let mut result = String::new();
                
                for line in lines {
                    if line.contains("![") && line.contains("](") {
                        // Find and replace image references
                        let mut new_line = line.to_string();
                        
                        // Look for image markdown pattern and extract filename
                        if let Some(start) = line.find("](") {
                            if let Some(end) = line[start+2..].find(")") {
                                let path_part = &line[start+2..start+2+end];
                                
                                // Check if it's just a filename (no path separators)
                                if !path_part.contains('/') && !path_part.contains('\\') && 
                                   (path_part.ends_with(".png") || path_part.ends_with(".jpg") || 
                                    path_part.ends_with(".jpeg") || path_part.ends_with(".gif") || 
                                    path_part.ends_with(".webp")) {
                                    
                                    let new_path = format!("{}/{}", relative_str, path_part);
                                    new_line = line.replace(&format!("]({})", path_part), &format!("]({})", new_path));
                                }
                            }
                        }
                        result.push_str(&new_line);
                    } else {
                        result.push_str(line);
                    }
                    result.push('\n');
                }
                
                return Ok(result);
            }
        }
    }
    
    // If no output path configured or empty (stdout), use absolute paths
    let abs_path = cfg.image_path.canonicalize()
        .unwrap_or_else(|_| cfg.image_path.clone())
        .to_string_lossy()
        .to_string();
    
    let lines: Vec<&str> = markdown.lines().collect();
    let mut result = String::new();
    
    for line in lines {
        if line.contains("![") && line.contains("](") {
            // Find and replace image references with absolute paths
            let mut new_line = line.to_string();
            
            // Look for image markdown pattern and extract filename
            if let Some(start) = line.find("](") {
                if let Some(end) = line[start+2..].find(")") {
                    let path_part = &line[start+2..start+2+end];
                    
                    // Check if it's just a filename (no path separators)
                    if !path_part.contains('/') && !path_part.contains('\\') && 
                       (path_part.ends_with(".png") || path_part.ends_with(".jpg") || 
                        path_part.ends_with(".jpeg") || path_part.ends_with(".gif") || 
                        path_part.ends_with(".webp")) {
                        
                        let new_path = format!("{}/{}", abs_path, path_part);
                        new_line = line.replace(&format!("]({})", path_part), &format!("]({})", new_path));
                    }
                }
            }
            result.push_str(&new_line);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    
    Ok(result)
}

fn extract_text_body(reader: &mut Reader<&[u8]>) -> Result<String, String> {
    let mut text_content = String::new();
    let mut buf = Vec::new();
    let mut current_paragraph = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(element)) => {
                if element.name().as_ref() == b"a:t" {
                    let text = extract_text_run(reader)?;
                    current_paragraph.push_str(&text);
                }
            }
            Ok(Event::End(element)) => {
                match element.name().as_ref() {
                    b"a:p" => {
                        if !current_paragraph.trim().is_empty() {
                            if is_title_text(&current_paragraph) {
                                text_content.push_str(&format!("### {}\n", current_paragraph.trim()));
                            } else {
                                text_content.push_str(&format!("- {}\n", current_paragraph.trim()));
                            }
                            current_paragraph.clear();
                        }
                    }
                    b"p:txBody" => break,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error extracting text body: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(text_content)
}

fn extract_text_run(reader: &mut Reader<&[u8]>) -> Result<String, String> {
    let mut text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(element)) => {
                if element.name().as_ref() == b"a:t" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error extracting text run: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(text)
}

fn extract_table(reader: &mut Reader<&[u8]>) -> Result<String, String> {
    let mut table = TableData { rows: vec![] };
    let mut buf = Vec::new();
    let mut current_row_index = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(element)) => {
                match element.name().as_ref() {
                    b"a:tr" => {
                        table.rows.push(vec![]);
                        current_row_index = table.rows.len() - 1;
                    }
                    b"a:tc" => {
                        let cell_content = extract_table_cell(reader)?;
                        if current_row_index < table.rows.len() {
                            table.rows[current_row_index].push(cell_content);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(element)) => {
                if element.name().as_ref() == b"a:tbl" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error extracting table: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(format_table_as_markdown(&table))
}

fn extract_table_cell(reader: &mut Reader<&[u8]>) -> Result<String, String> {
    let mut cell_content = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                cell_content.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(element)) => {
                if element.name().as_ref() == b"a:tc" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error extracting table cell: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(cell_content.trim().to_string())
}

fn format_table_as_markdown(table: &TableData) -> String {
    if table.rows.is_empty() {
        return String::new();
    }

    let mut markdown = String::new();

    // Header row
    if !table.rows.is_empty() {
        markdown.push('|');
        for cell in &table.rows[0] {
            markdown.push_str(&format!(" {} |", cell));
        }
        markdown.push('\n');

        // Separator row
        markdown.push('|');
        for _ in &table.rows[0] {
            markdown.push_str("---|");
        }
        markdown.push('\n');

        // Data rows
        for row in table.rows.iter().skip(1) {
            markdown.push('|');
            for cell in row {
                markdown.push_str(&format!(" {} |", cell));
            }
            markdown.push('\n');
        }
    }

    markdown
}

fn is_title_text(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() < 100 && 
    !trimmed.ends_with('.') && 
    !trimmed.ends_with('!') && 
    !trimmed.ends_with('?') &&
    !trimmed.contains('\n')
}
