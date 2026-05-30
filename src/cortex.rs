use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct MiniMaxRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct MiniMaxResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

pub struct Cortex {
    client: Client,
    api_key: String,
    model: String,
}

impl Cortex {
    pub fn new(model: &str) -> Self {
        let api_key = env::var("MINIMAX_API_KEY").unwrap_or_else(|_| "MISSING_KEY".to_string());
        Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
        }
    }

    pub async fn think(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        if self.api_key == "MISSING_KEY" {
            return Ok("Hiba: Hiányzik a MINIMAX_API_KEY környezeti változó!".to_string());
        }

        let url = "https://api.minimax.chat/v1/text/chatcompletion_v2";

        let req = MiniMaxRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "Te vagy Rongyász Agent v2.0, egy autonóm kódsebész. Válaszolj tömören, magyarul.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let res = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let body: MiniMaxResponse = res.json().await?;
        
        if let Some(choice) = body.choices.get(0) {
            return Ok(choice.message.content.clone());
        }

        Ok("Nem érkezett válasz az API-tól.".to_string())
    }
}
