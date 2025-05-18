use reqwest::blocking::Client;
use serde_json::{json, Value};

fn main() {
    let api_key = "YOUR_API_KEY";
    let client = Client::new();

    // 你的图片 URL（必须是公开可访问的）
    let img_url = "https://dashscope.oss-cn-beijing.aliyuncs.com/images/dog_and_girl.jpeg";
    let prompt = "给这个图片起一个标题";

    // 构造请求体，messages数组里有两个消息，一个是图片，一个是文本
    let body = json!({
        "model": "gpt-4o",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": img_url
                        }
                    }
                ]
            },
            {
                "role": "system",
                "content": [
                    {
                        "type": "text",
                        "text": prompt
                    }
                ]
            }
        ]
    });

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .expect("请求失败");

    let text = response.text().expect("读取响应失败");
    println!("响应原始文本：\n{}", text);

    let json_result: Result<Value, _> = serde_json::from_str(&text);
    if let Ok(json) = json_result {
        if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
            println!("模型回答：\n{}", content);
        } else {
            eprintln!("未能提取模型回答！");
        }
    } else {
        eprintln!("响应不是合法的 JSON！");
    }
}
