use anyhow::{bail, Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "PDF to DOCX Converter",
    long_about = "Converts PDF files to DOCX format using Python's pdf2docx library"
)]
struct CliArgs {
    /// Input PDF file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output DOCX file path
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    validate_input_file(&args.input)?;
    prepare_output_directory(&args.output)?;
    check_python_environment()?;
    check_pdf2docx_installation()?;
    convert_pdf_to_docx(&args.input, &args.output)?;

    println!("Conversion successful!");
    Ok(())
}

fn validate_input_file(input_path: &Path) -> Result<()> {
    if !input_path.exists() {
        bail!("Input file does not exist: {}", input_path.display());
    }
    if input_path.extension().and_then(|s| s.to_str()) != Some("pdf") {
        bail!("Input file must be a PDF file");
    }
    Ok(())
}

fn prepare_output_directory(output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    Ok(())
}

fn check_python_environment() -> Result<()> {
    let python_commands = ["python3", "python"];
    let mut found = false;

    for cmd in python_commands {
        if Command::new(cmd)
            .arg("--version")
            .output()
            .is_ok()
        {
            found = true;
            break;
        }
    }

    if !found {
        bail!("Python not found. Please install Python 3.7+ and ensure it's in your PATH");
    }
    Ok(())
}

fn check_pdf2docx_installation() -> Result<()> {
    let status = Command::new("python")
        .args(["-c", "import pdf2docx"])
        .status()
        .or_else(|_| Command::new("python3").args(["-c", "import pdf2docx"]).status())
        .context("Failed to check pdf2docx installation")?;

    if !status.success() {
        bail!(
            "pdf2docx library not found. Please install with:\n\tpip install pdf2docx"
        );
    }
    Ok(())
}

fn convert_pdf_to_docx(input: &Path, output: &Path) -> Result<()> {
    let python_script = format!(
        r#"
from pdf2docx import Converter
import sys

try:
    cv = Converter(r"{}")
    cv.convert(r"{}")
    cv.close()
    print("Conversion completed successfully")
except Exception as e:
    print(f"Conversion failed: {{e}}")
    sys.exit(1)
"#,
        input.display(),
        output.display()
    );

    let output = Command::new("python")
        .args(["-c", &python_script])
        .output()
        .or_else(|_| Command::new("python3").args(["-c", &python_script]).output())
        .context("Failed to execute conversion script")?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Conversion failed:\n{}",
            error_message.trim().is_empty().then(|| String::from_utf8_lossy(&output.stdout)).unwrap_or(error_message)
        );
    }

    Ok(())
}
