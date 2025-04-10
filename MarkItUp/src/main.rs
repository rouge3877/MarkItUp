use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufWriter, Write};
mod model;
use model::XmlToMarkdown::XmlToMarkdownConverter;

fn main() {
    let xml = r#"
    <doc>
        <title>Rust XML to Markdown</title>
        
        <section>
            <header>Getting Started</header>
            <para>Rust is a systems programming language.</para>
        </section>

        <section>
            <header>Features</header>
            <list>
                <item>Memory safety</item>
                <item>Concurrency</item>
                <item>Zero-cost abstractions</item>
            </list>
        </section>

        <section>
            <header>Example Code</header>
            <code>fn main() {
                    println!("Hello, world!");
                }</code>
        </section>

        <section>
            <header>Text Styling</header>
            <para>This is a <bold>bold</bold> word and this is an <italic>italic</italic> word.</para>
        </section>

        <section>
            <header>Useful Link</header>
            <para>Visit the <link>official Rust website</link> for more info.</para>
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
