use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufWriter, Write};
mod model;
use model::XmlToMarkdown::XmlToMarkdownConverter;

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

    let markdown = XmlToMarkdownConverter::convert_string(xml).expect("convert failed");

    let output_file = File::create("output.md").expect("Unable to create file");
    let mut writer = BufWriter::new(output_file);
    writer
        .write_all(markdown.as_bytes())
        .expect("Unable to write to file");

    println!("Markdown output written to output.md");
}
