use std::env;
use std::fs;
use std::fs::File;
use anyhow::{Context, Result};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;

fn docx_to_xml_pandoc(input_path: &str, output_path: &str) -> Result<()> {
    Command::new("pandoc")
        .args(&[
            input_path,
            "-f", "docx",
            "-t", "jats",
            "-o", output_path,
            "--extract-media=./images"
        ])
        .status()
        .context("Failed to execute Pandoc")?;

    Ok(())
}

fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Check if the input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file does not exist.");
        std::process::exit(1);
    }

    docx_to_xml_pandoc(input_path, output_path);
    println!("Conversion successful! Output saved to {}", output_path);

    Ok(())
}

