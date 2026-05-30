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

/// Típusbiztos parancs enum — az LLM soha nem írhat nyers adatot a Spine-ra.
/// A Cortex csak ezen a Guard rétegen keresztül adhat ki parancsot.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentCommand {
    WriteMessage(String),   // max 240 byte, validált
    QueryStatus,
    Noop,
    Invalid(String),        // parse hiba — loggolva, de eldobva
}

impl AgentCommand {
    /// Parsolja az LLM szöveges kimenetét típusbiztos parancsra.
    /// Csak ismert prefix-ek fogadottak el — minden más Noop vagy Invalid lesz.
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return AgentCommand::Noop;
        }

        // Strukturált parancs: "CMD:WRITE:<üzenet>"
        if let Some(rest) = trimmed.strip_prefix("CMD:WRITE:") {
            let msg = rest.trim().to_string();
            if msg.is_empty() || msg.len() > 240 {
                return AgentCommand::Invalid(format!("WRITE payload méret hiba: {} byte", msg.len()));
            }
            // Csak printable ASCII + UTF-8 magyar karakterek
            if msg.chars().any(|c| c.is_control() && c != '\n') {
                return AgentCommand::Invalid("WRITE payload control karaktert tartalmaz".to_string());
            }
            return AgentCommand::WriteMessage(msg);
        }

        if trimmed == "CMD:STATUS" {
            return AgentCommand::QueryStatus;
        }

        if trimmed == "CMD:NOOP" {
            return AgentCommand::Noop;
        }

        // Szabad szöveges válasz — nem parancs, hanem információ
        // Nincs Spine írás, csak logolás
        AgentCommand::Noop
    }

    pub fn is_executable(&self) -> bool {
        matches!(self, AgentCommand::WriteMessage(_) | AgentCommand::QueryStatus)
    }
}

pub struct Cortex {
    client: Client,
    api_key: String,
    model: String,
}

impl Cortex {
    pub fn new(model: &str) -> Self {
        let api_key = env::var("MINIMAX_API_KEY")
            .unwrap_or_else(|_| "MISSING_KEY".to_string());
        Self {
            client: Client::new(),
            api_key,
            model: model.to_string(),
        }
    }

    /// Nyers LLM választ ad vissza — a hívónak kell AgentCommand::parse()-olni
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
                    content: concat!(
                        "Te vagy Rongyász Agent v2.0, egy autonóm kódsebész. ",
                        "Ha parancsot akarsz kiadni, KIZÁRÓLAG az alábbi formátumokat használd:\n",
                        "CMD:WRITE:<üzenet>  — max 240 karakter\n",
                        "CMD:STATUS          — státusz lekérdezés\n",
                        "CMD:NOOP            — nincs teendő\n",
                        "Minden más szöveges válasz információ, nem parancs. Válaszolj tömören, magyarul."
                    ).to_string(),
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

        if let Some(choice) = body.choices.first() {
            return Ok(choice.message.content.clone());
        }

        Ok("Nem érkezett válasz az API-tól.".to_string())
    }

    /// think() + Guard parse — csak AgentCommand-ot ad vissza
    pub async fn think_command(&self, prompt: &str) -> Result<AgentCommand, Box<dyn std::error::Error>> {
        let raw = self.think(prompt).await?;
        let cmd = AgentCommand::parse(&raw);
        if let AgentCommand::Invalid(ref reason) = cmd {
            eprintln!("[CORTEX GUARD] Érvénytelen parancs eldobva: {}", reason);
        }
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_write_command() {
        let cmd = AgentCommand::parse("CMD:WRITE:Szia Máté!");
        assert_eq!(cmd, AgentCommand::WriteMessage("Szia Máté!".to_string()));
    }

    #[test]
    fn parse_status_command() {
        assert_eq!(AgentCommand::parse("CMD:STATUS"), AgentCommand::QueryStatus);
    }

    #[test]
    fn parse_noop() {
        assert_eq!(AgentCommand::parse("CMD:NOOP"), AgentCommand::Noop);
        assert_eq!(AgentCommand::parse("   "), AgentCommand::Noop);
    }

    #[test]
    fn free_text_is_noop() {
        let cmd = AgentCommand::parse("Ez egy szöveges válasz, nem parancs.");
        assert_eq!(cmd, AgentCommand::Noop);
    }

    #[test]
    fn oversized_payload_is_invalid() {
        let long = "x".repeat(241);
        let raw = format!("CMD:WRITE:{}", long);
        assert!(matches!(AgentCommand::parse(&raw), AgentCommand::Invalid(_)));
    }

    #[test]
    fn control_char_is_invalid() {
        let cmd = AgentCommand::parse("CMD:WRITE:hello\x01world");
        assert!(matches!(cmd, AgentCommand::Invalid(_)));
    }

    #[test]
    fn executable_check() {
        assert!(AgentCommand::WriteMessage("x".to_string()).is_executable());
        assert!(AgentCommand::QueryStatus.is_executable());
        assert!(!AgentCommand::Noop.is_executable());
    }
}
