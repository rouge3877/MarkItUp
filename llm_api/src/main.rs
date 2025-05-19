use reqwest::blocking::Client;
use serde_json::json;
use serde_json::Value;

fn main() {
    let api_key = "sk-251c110d2ce44b5b93e3618d3df0dc90";
    let client = Client::new();

    let input_text = "Rust 是一种系统编程语言，专注于安全性、并发性和性能。";

    let prompt = format!("总结一下这段文本：{}", input_text);

    let body = json!({
        "model": "deepseek-chat",
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let response = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .unwrap();

    let text = response.text().unwrap();

    let json: Value = serde_json::from_str(&text).unwrap();

    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        println!("模型回答：\n{}", content);
    } else {
        eprintln!("未能提取模型回答！");
    }
}
