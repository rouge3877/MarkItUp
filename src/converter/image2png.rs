use image::ImageFormat;
use std::io::Cursor;

/// Convert various image formats to PNG format
/// 
/// # Arguments
/// * `input_bytes` - Raw bytes of the input image
/// 
/// # Returns
/// * `Ok(Vec<u8>)` - PNG encoded bytes on success
/// * `Err(String)` - Error message on failure
/// 
/// # Supported formats
/// * JPEG/JPG
/// * GIF
/// * BMP
/// * TIFF
/// * WebP
/// * ICO
/// * PNG (passthrough with potential optimization)
pub fn image_to_png(input_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // Create a cursor from input bytes
    let cursor = Cursor::new(input_bytes);
    
    // Try to guess the format and decode the image
    let img = image::load(cursor, ImageFormat::from_mime_type("image/*").unwrap_or(ImageFormat::Png))
        .or_else(|_| {
            // If auto-detection fails, try common formats explicitly
            let formats = [
                ImageFormat::Jpeg,
                ImageFormat::Png,
                ImageFormat::Gif,
                ImageFormat::WebP,
                ImageFormat::Bmp,
                ImageFormat::Tiff,
                ImageFormat::Ico,
            ];
            
            for format in &formats {
                let cursor = Cursor::new(input_bytes);
                if let Ok(img) = image::load(cursor, *format) {
                    return Ok(img);
                }
            }
            
            Err(image::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::Format(image::error::ImageFormatHint::Unknown),
                )
            ))
        })
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    // Convert to PNG format
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;
    
    Ok(png_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_png_passthrough() {
        // Create a simple 1x1 PNG image
        let img = image::RgbImage::new(1, 1);
        let dynamic_img = image::DynamicImage::ImageRgb8(img);
        
        let mut original_png = Vec::new();
        let mut cursor = Cursor::new(&mut original_png);
        dynamic_img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        
        let result = image_to_png(&original_png).unwrap();
        assert!(!result.is_empty());
        
        // Verify the result is valid PNG
        let cursor = Cursor::new(&result);
        assert!(image::load(cursor, ImageFormat::Png).is_ok());
    }
    
    #[test]
    fn test_invalid_image_data() {
        let invalid_data = b"not an image";
        let result = image_to_png(invalid_data);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_input() {
        let empty_data = &[];
        let result = image_to_png(empty_data);
        assert!(result.is_err());
    }
}