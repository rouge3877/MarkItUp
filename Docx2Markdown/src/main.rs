use anyhow::{Context, Result};
use std::process::Command;
use std::fs::File;
use std::io::{Read, Write};


fn docx_to_md_pandoc(input_path: &str, output_path: &str) -> Result<()> {
    Command::new("pandoc")
        .args(&[
            input_path,
            "-o", output_path,
            "-t", "gfm",
            "--standalone",
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

    docx_to_md_pandoc(input_path, output_path);
    println!("Conversion successful! Output saved to {}", output_path);

    Ok(())
}

