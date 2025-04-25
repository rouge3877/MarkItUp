use pyo3::prelude::*;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let code = r#"
audio_path = "audio/example.mp3"
markdown_content = f"[点击播放音频]({audio_path})"

with open("audio_link.md", "w", encoding="utf-8") as f:
    f.write(markdown_content)

print("Markdown 文件已生成，内容如下：")
print(markdown_content)
"#;
        py.run(code, None, None)?;
        Ok(())
    })
}
