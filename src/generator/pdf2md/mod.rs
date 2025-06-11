use std::error::Error;

use lopdf::Document;
use pdf_element::{PdfElement, PdfText, PdfUnit};
use pdf_page::PdfPage;


mod pdf_element;
mod pdf_page;
mod pdf_state;

/// Convert PDF byte stream into markdown
pub fn run(file_stream: &[u8]) -> Result<String, String> {
    let pdf = Pdf::new_from_bytes(file_stream)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;
    let mut result = String::new();
    let mut i = 0;

    for page in pdf.iter_pages() {
        i += 1;
        result.push_str(&format!("\n\n<!-- S-TITLE: Page number {} -->\n", i));
        let mut page = page.map_err(|e| format!("Failed to process page {}: {}", i, e))?;
        let units = page.handle_stream(page.stream.clone())
            .map_err(|e| format!("Failed to handle stream for page {}: {}", i, e))?;
        let font_sizes: Vec<f32> = units
            .iter()
            .filter_map(|u| match u {
                PdfUnit::Text(pdf_text) => pdf_text.font_size,
                PdfUnit::Line(_) => None,
            })
            .collect();
        let median_font_size = median(font_sizes);
        let elements = Pdf::pdf_units_to_elements(units);
        let mut pre_header_level = "";

        for row in elements {
            let mut current_header_level = "";
            for e in row {
                match e {
                    pdf_element::PdfElement::Text(pdf_text) => {
                        let (mut text, header_level) = pdftext_to_md(pdf_text, median_font_size);
                        current_header_level = header_level;
                        // connected headres
                        if current_header_level != "" && pre_header_level == current_header_level {
                            result.push_str("<br>");
                            text = text.replace(header_level, "");
                        }
                        // used to be header, now different
                        if pre_header_level != "" && current_header_level != pre_header_level {
                            result.push_str("\n");
                        }
                        result.push_str(&format!("{} ", text));
                    }
                    pdf_element::PdfElement::Table(mut pdf_table) => {
                        let elements = pdf_table.get_sorted_elements();
                        let elements: Vec<Vec<String>> = elements
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|cell| {
                                        cell.iter()
                                            // never header
                                            .map(|item| pdftext_to_md(item.clone(), Some(1000.0)).0)
                                            .collect::<Vec<String>>()
                                            .join(" ")
                                    })
                                    .collect()
                            })
                            .collect();

                        let headers = elements.get(0);
                        let rows = elements.get(1..);
                        if rows.is_some() && headers.is_some() {
                            let md = to_markdown_table(&headers.unwrap(), &rows.unwrap());
                            result.push_str(&md);
                        }
                    }
                }
            }
            if current_header_level == "" {
                result.push_str("\n");
            }
            pre_header_level = current_header_level;
        }
    }

    Ok(result)
}

fn pdftext_to_md(mut unit: PdfText, median_size: Option<f32>) -> (String, &'static str) {
    let mut text = unit.text;

    if let Some(color) = unit.color {
        if color != "#FFFFFF" {
            text = format!("`{}` ", text);
        }
    }
    if let Some(name) = unit.font_name {
        let lwc = name.to_lowercase();
        if lwc.contains("bold") {
            text = format!("**{}** ", text.trim());
        }
        if lwc.contains("italic") {
            unit.italic = true;
        }
    }
    if unit.italic {
        text = format!("*{}* ", text.trim());
    }
    if unit.underlined {
        text = format!("<u>{}</u> ", text.trim());
    }

    let mut is_header = "";
    if let Some(header) = font_size_to_header(unit.font_size.unwrap_or_default(), median_size) {
        text = format!("{header} {} ", text.trim());
        is_header = header;
    }

    return (text, is_header);
}

fn font_size_to_header(font_size: f32, median_size: Option<f32>) -> Option<&'static str> {
    let base_size = median_size.unwrap_or(12.0);
    let size_ratio = font_size / base_size;

    match size_ratio {
        ratio if ratio >= 3.0 => Some("#"), // 50%+ larger (H1)
        ratio if ratio >= 2.5 && ratio < 3.0 => Some("##"), // 30-50% larger (H2)
        ratio if ratio >= 2.0 && ratio < 2.5 => Some("###"), // 20-30% larger (H3)
        _ => None,                          // Equal to or smaller than base size is regular text
    }
}

struct Pdf {
    doc: Document,
}

impl Pdf {
    fn new_from_bytes(bytes: &[u8]) -> Result<Pdf, Box<dyn Error>> {
        let doc = lopdf::Document::load_from(bytes)?;
        let pdf = Pdf { doc };

        Ok(pdf)
    }

    pub fn iter_pages(&self) -> impl Iterator<Item = Result<PdfPage, Box<dyn Error>>> {
        self.doc
            .page_iter()
            .map(|id| PdfPage::from_object_id(&self.doc, id))
    }

    pub fn pdf_units_to_elements(units: Vec<PdfUnit>) -> Vec<Vec<PdfElement>> {
        let elements = pdf_element::units_to_elements(units);
        let mut matrix = pdf_element::elements_into_matrix(elements);
        for row in matrix.iter_mut() {
            pdf_element::sort_transform_elements(row);
        }
        matrix
    }
}

fn median(mut values: Vec<f32>) -> Option<f32> {
    let len = values.len();
    if len == 0 {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    Some(if len % 2 == 1 {
        values[len / 2]
    } else {
        (values[len / 2 - 1] + values[len / 2]) / 2.0
    })
}

fn to_markdown_table(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut output = String::new();
    output += &format!("| {} |\n", headers.join(" | "));
    output += &format!("|{}|\n", vec!["---"; headers.len()].join("|"));

    for row in rows {
        output += &format!("| {} |\n", row.join(" | "));
    }

    output
}