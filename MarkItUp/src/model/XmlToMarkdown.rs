use quick_xml::events::Event;
use quick_xml::Reader;
use std::error::Error;

pub struct XmlToMarkdownConverter;

impl XmlToMarkdownConverter {
    pub fn convert_string(xml_content: &str) -> Result<String, Box<dyn Error>> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        Self::convert(&mut reader)
    }

    fn convert<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<String, Box<dyn Error>> {
        let mut buf = Vec::new();
        let mut markdown = String::new();
        let mut current_tag = String::new();
        let mut in_list = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match current_tag.as_str() {
                        "section" => markdown.push_str("\n"),
                        "header" => markdown.push_str("## "),
                        "list" => in_list = true,
                        "code" => markdown.push_str("```\n"),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag.as_str() {
                        "code" => markdown.push_str("```\n\n"),
                        "list" => {
                            in_list = false;
                            markdown.push('\n');
                        }
                        "header" | "section" => markdown.push_str("\n\n"),
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    let line = match current_tag.as_str() {
                        "title" => format!("# {}\n\n", text),
                        "header" => format!("{}\n\n", text),
                        "section" => format!("{}\n\n", text),
                        "para" => format!("{}\n\n", text),
                        "bold" | "b" => format!("**{}**", text),
                        "italic" | "i" => format!("*{}*", text),
                        "item" if in_list => format!("- {}\n", text),
                        "code" => format!("{}\n", text),
                        "link" => format!("[{}](#)", text),
                        _ => format!("{}\n\n", text),
                    };
                    markdown.push_str(&line);
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Box::new(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(markdown)
    }
}
