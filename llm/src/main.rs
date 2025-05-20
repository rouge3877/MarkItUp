use std::fs;
use std::env;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMessage {
    pub model: String,
    pub messages: Vec<Message>,
    pub presence_penalty: f32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ROLE {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum MessageContent {
    ImageUrl([ImageData; 1]),
    PlainText(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: ROLE,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageData {
    pub r#type: String,
    pub image_url: String,
}

impl Message {
    pub fn new(role: ROLE, content: MessageContent) -> Self {
        Self { role, content }
    }

    pub fn new_text(role: ROLE, text: String) -> Self {
        Self {
            role,
            content: MessageContent::PlainText(text),
        }
    }
}

impl SendMessage {
    pub fn new(
        model: String,
        presence_penalty: Option<f32>,
        temperature: Option<f32>,
    ) -> Self {
        let mut messages = Vec::new();
        messages.push(Message::new_text(
            ROLE::System,
            "这里添加你要的 system prompt".to_string(),
        ));

        Self {
            model,
            messages,
            presence_penalty: presence_penalty.unwrap_or(0.0),
            temperature: temperature.unwrap_or(1.0),
        }
    }

    pub fn add_system_message(&mut self, content: String) {
        let mut count = 0;
        for msg in &self.messages {
            if msg.role == ROLE::System {
                count += 1;
            } else {
                break;
            }
        }
        self.messages
            .insert(count, Message::new_text(ROLE::System, content));
    }

    pub fn extend_message(&mut self, vec: Vec<Message>) {
        self.messages.extend(vec);
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub choices: Vec<Choice>,
    pub created: u64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub finish_reason: String,
    pub index: u64,
    pub logprobs: Option<serde_json::Value>,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub prompt_tokens_details: PromptTokensDetails,
    pub total_tokens: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptTokensDetails {
    pub cached_tokens: u64,
}

pub async fn send_api_post(
    client: &Client,
    url: &str,
    payload: &impl serde::Serialize,
) -> Result<Response, Box<dyn std::error::Error>> {
    let key = env::var("DOUBAO_API_KEY")
        .expect("请先设置环境变量 DOUBAO_API_KEY");
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", key))
        .header("Accept", "application/json")
        .json(payload)
        .send()
        .await?;

    let raw = res.text().await?;
    println!("Raw Response: {}", raw);

    let response: Response = serde_json::from_str(&raw)?;
    Ok(response)
}

fn image_to_base64(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let image_bytes = fs::read(path)?;
    let mime = if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };
    let encoded = general_purpose::STANDARD.encode(image_bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";

    let mut request = SendMessage::new(
        "doubao-1-5-thinking-vision-pro-250428".to_string(),    //这里写模型 ID
        None,
        None,
    );
    request.add_message(Message::new_text(
        ROLE::User,
        "请分析这张图片的内容".to_string(),
    ));

    let base64_image_url = image_to_base64("D:/rust/MarkItUp/AA1EiWym.jpg")?;
    let image_msg = Message::new(
        ROLE::User,
        MessageContent::ImageUrl([ImageData {
            r#type: "image_url".to_string(),
            image_url: base64_image_url,
        }]),
    );
    request.add_message(image_msg);

    let response = send_api_post(&client, url, &request).await?;
    println!("Parsed Response: {:#?}", response);

    Ok(())
}

//运行方式: $Env:DOUBAO_API_KEY=""
>> cargo run
