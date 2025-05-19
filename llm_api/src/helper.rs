use serde::{Serialize, Deserialize};
use serde_json::json;
use reqwest::Client;
#[derive(Serialize,Deserialize,Debug)]
pub struct SendMessage{
  pub model: String,
  messages: Vec<Message>,
  presence_penalty: f32, // 介于-2 ~ 2之间，越大越容易转移话题
  temperature: f32, // 介于0 ~ 2之间，越大越随机
}
#[derive(Serialize,Deserialize,Debug,PartialEq)]
#[serde(rename_all = "lowercase")] // 将枚举值序列化为小写字符串
pub enum ROLE{
  System,
  User,
  Assistant,
}

#[derive(Serialize,Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum MessageContent{
  ImageUrl([ImageData;1]),
  PlainText(String),
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Message{
  pub role: ROLE,
  pub content: MessageContent,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct ImageData {
  pub r#type: String,
  pub image_url: String, //可以是网址，可以是Base64
}

impl Message{
  pub fn new(role: ROLE, content: MessageContent)->Self{
    Self{
      role,
      content,
    }
  }

  pub fn new_text(role: ROLE, text: String)->Self{
    Self{
      role,
      content: MessageContent::PlainText(text),
    }
  }
}

impl SendMessage{
  pub fn new(model: String, presence_penalty: Option<f32>, temperature: Option<f32>)->Self{
    let mut message = Vec::new();
    message.push(Message::new_text(ROLE::System, "这里添加你要的system prompt".to_string()));

    Self{
      model,
      messages: message,
      presence_penalty: presence_penalty.unwrap_or(0.0),
      temperature: temperature.unwrap_or(1.0),
    }
  }

  pub fn add_system_message(&mut self, content: String){
    let mut count:usize = 0;
    for i in self.messages.iter(){
      if i.role == ROLE::System{
        count += 1;
      }else{
        break;
      }
    }
    self.messages.insert(count, Message::new_text(ROLE::System, content));
  }

  pub fn extend_message(&mut self, vec: Vec<Message>){
    self.messages.extend(vec);
  }

  pub fn add_message(&mut self, message: Message){
    self.messages.push(message);
  }
}


#[derive(Serialize,Deserialize,Debug)]
pub struct Response{
  choices: Vec<Choice>,
  created: u64,
  id: String,
  model: String,
  object: String,
  pub usage: Usage,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Choice{
  finish_reason: String,
  index: u64,
  logprobs: Option<serde_json::Value>,
  message: Message,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Usage{
  pub completion_tokens: u64,
  pub prompt_tokens: u64,
  pub prompt_tokens_details: PromptTokensDetails,
  pub total_tokens: u64,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct PromptTokensDetails{
  pub cached_tokens: u64,
}




pub async fn send_api_post(client:&Client, url: &str, payload: &impl serde::Serialize) -> Result<Response, Box<dyn std::error::Error>>{
  let key: &str = "模型的key";
  let res = client.post(url)
    .header("Content-Type", "application/json") 
    .header("Authorization", "Bearer ".to_string() + key) 
    .header("Accept", "application/json")
    .json(&json!(payload))
    .send()
    .await?;
  let response = res.json::<Response>().await?;
  println!("Response: {:?}", response);
  Ok(response)
}

// #[tokio::main]
// async fn main()-> Result<(), Box<dyn std::error::Error>>{
//   let client = Client::new();
//   let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
//   let mut request = SendMessage::new("doubao-1.5-vision-pro-32k-250115".to_string(), None, None);
//   //后面加你要的上下文
//   let response = send_api_post(&client, url, &request).await?;
//   println!("Response: {:?}", response);
//   Ok(())
//   //详情见官网示例：https://www.volcengine.com/docs/82379/1494384?redirect=1#%E8%AF%B7%E6%B1%82%E7%A4%BA%E4%BE%8B
// }