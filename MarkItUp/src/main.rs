use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufWriter, Write};

fn xml_to_markdown(xml_content: &str) -> String {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut markdown = String::new();
    let mut current_tag = String::new();

    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Start(e) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Event::Text(e) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "title" => markdown.push_str(&format!("# {}\n\n", text)),
                    "header" => markdown.push_str(&format!("## {}\n\n", text)),
                    "para" => markdown.push_str(&format!("{}\n\n", text)),
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    markdown
}

fn main() {
    let xml = r#"
    <doc>
        <title>Hello World</title>
        <section>
            <header>Introduction</header>
            <para>This is an example paragraph.</para>
        </section>
    </doc>
    "#;

    let markdown = xml_to_markdown(xml);

    let output_file = File::create("output.md").expect("Unable to create file");
    let mut writer = BufWriter::new(output_file);
    writer
        .write_all(markdown.as_bytes())
        .expect("Unable to write to file");

    println!("Markdown output written to output.md");
}
