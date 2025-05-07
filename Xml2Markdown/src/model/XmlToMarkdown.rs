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
        let mut is_ordered_list = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match current_tag.as_str() {
                        "section" => markdown.push_str("\n"),
                        "header" => markdown.push_str("## "),
                        "list" => in_list = true,
                        "code" => markdown.push_str("```\n"),
                        "underline" => markdown.push_str("<u>"),
                        "link" => {
                            if let Some(href) = e.attributes().find_map(|a| {
                                a.ok().and_then(|a| {
                                    if a.key.as_ref() == b"href" {
                                        Some(String::from_utf8_lossy(&a.value).to_string())
                                    } else {
                                        None
                                    }
                                })
                            }) {
                                current_tag = format!("link:{}", href);
                            }
                        }
                        "subheader" => markdown.push_str("### "),
                        "strong" => markdown.push_str("**"),
                        "em" => markdown.push_str("*"),
                        "strike" => markdown.push_str("~~"),
                        "blockquote" => markdown.push_str("> "),
                        "hr" => markdown.push_str("\n---\n"),
                        "inline-code" => markdown.push_str("`"),
                        "pre" => markdown.push_str("```\n"),
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
                        "underline" => markdown.push_str("</u>"),
                        "strong" => markdown.push_str("**"),
                        "em" => markdown.push_str("*"),
                        "strike" => markdown.push_str("~~"),
                        "inline-code" => markdown.push_str("`"),
                        "pre" => markdown.push_str("```\n\n"),
                        _ => {}
                    }
                    current_tag.clear();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    let line = if current_tag.starts_with("link:") {
                        let href = current_tag.trim_start_matches("link:");
                        format!("[{}]({})", text, href)
                    } else {
                        match current_tag.as_str() {
                            "title" => format!("# {}\n\n", text),
                            "header" => format!("{}\n\n", text),
                            "section" => format!("{}\n\n", text),
                            "para" => format!("{}\n\n", text),
                            "bold" | "b" => format!("**{}**", text),
                            "italic" | "i" => format!("*{}*", text),
                            "underline" => format!("{}", text),
                            "item" if in_list => format!("- {}\n", text),
                            "code" => format!("{}\n", text),
                            "item" if in_list && is_ordered_list => format!("1. {}\n", text),
                            "inline-code" => format!("{}", text),
                            "blockquote" => format!("{}\n", text),
                            _ => format!("{}\n\n", text),
                        }
                    };
                    markdown.push_str(&line);
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"a" => {
                    if let Some(href) = e.attributes().find_map(|a| {
                        a.ok().and_then(|a| if a.key.as_ref() == b"href" {
                            Some(String::from_utf8_lossy(&a.value).to_string())
                        } else {
                            None
                        })
                    }) {
                        current_tag = format!("link:{}", href);
                    }
                }
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"img" => {
                    let mut src = String::new();
                    let mut alt = String::new();

                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"src" => src = String::from_utf8_lossy(&attr.value).to_string(),
                            b"alt" => alt = String::from_utf8_lossy(&attr.value).to_string(),
                            _ => {}
                        }
                    }
                    markdown.push_str(&format!("![{}]({})\n\n", alt, src));
                }
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"graphic" => {
                    let mut href = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"xlink:href" {
                            href = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !href.is_empty() {
                        markdown.push_str(&format!("![]({})\n\n", href));
                    }
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
