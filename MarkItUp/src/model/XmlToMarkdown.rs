use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufWriter, Write,BufReader};
use std::path::Path;
pub struct XmlToMarkdownConverter;
impl XmlToMarkdownConverter{
    pub fn convert_file<P:AsRef<Path>>(path:P) ->Result<String, Box<dyn std::error::Error>>{
        let file = File::open(path)?;
        let file_reader =BufReader::new(file);
        let mut reader= Reader::from_reader(file_reader);
        reader.trim_text(true);
    
        let mut buf = Vec::new();
        let mut markdown = String::new();
        let mut current_tag = String::new();
        loop{
            match reader.read_event_into(&mut buf){
                Ok(Event::Start(e))=>
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string(),
                Ok(Event::Text(e))=>{
                    let text = e.unescape()?.to_string();
                    match current_tag.as_str(){
                        "title" => markdown.push_str(&format!("# {}\n\n", text)),
                        "para" => markdown.push_str(&format!("{}\n\n", text)),
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) =>return Err(Box::new(e)),
                _ => {}

            }
            buf.clear();
        }
        Ok(markdown)
    }   
    pub fn convert_string(xml_content:&str) ->Result<String ,Box<dyn std::error::Error>>{
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut markdown = String::new();
        let mut current_tag = String::new();
        loop{
            match reader.read_event_into(&mut buf){
                Ok(Event::Start(e))=>
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string(),
                Ok(Event::Text(e))=>{
                    let text = e.unescape()?.to_string();
                    match current_tag.as_str(){
                        "title" => markdown.push_str(&format!("# {}\n\n", text)),
                        "para" => markdown.push_str(&format!("{}\n\n", text)),
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) =>return Err(Box::new(e)),
                _ => {}

            }
            buf.clear();
        }
        Ok(markdown)

    }
}
