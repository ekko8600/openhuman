use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_DEEPSEEK_API_BASE: &str = "https://api.deepseek.com";
const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-v4-pro";

#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
}

impl DeepSeekConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .context("DEEPSEEK_API_KEY is required for literature summary/analyze commands")?;
        let api_base = std::env::var("DEEPSEEK_API_BASE")
            .unwrap_or_else(|_| DEFAULT_DEEPSEEK_API_BASE.to_string());
        let model =
            std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| DEFAULT_DEEPSEEK_MODEL.to_string());
        Ok(Self {
            api_key,
            api_base,
            model,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

pub fn deepseek_chat(config: &DeepSeekConfig, messages: Vec<ChatMessage>) -> Result<String> {
    let url = format!("{}/chat/completions", config.api_base.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&ChatCompletionRequest {
            model: config.model.clone(),
            messages,
            temperature: 0.2,
        })
        .send()
        .context("failed to call DeepSeek chat completions API")?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        bail!("DeepSeek API returned {status}: {body}");
    }

    let body: ChatCompletionResponse = response.json().context("invalid DeepSeek API response")?;
    body.choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .context("DeepSeek API response did not include assistant content")
}

pub fn summary_messages(
    title: Option<&str>,
    language: &str,
    focus: Option<&str>,
    chunk_context: &str,
) -> Vec<ChatMessage> {
    let title = title.unwrap_or("Untitled document");
    let focus = focus.unwrap_or("general literature understanding");
    vec![
        ChatMessage {
            role: "system".into(),
            content: format!(
                "You are a rigorous research assistant. Produce a {language} literature analysis grounded only in the provided chunks. Preserve cited chunk IDs."
            ),
        },
        ChatMessage {
            role: "user".into(),
            content: format!(
                "Analyze this paper for {focus}.\n\nTitle: {title}\n\nRequired Markdown sections:\n- TL;DR\n- Research problem\n- Method\n- Key contributions\n- Evidence / experiments\n- Limitations\n- Follow-up questions\n- Cited chunks\n\nLocal chunks:\n{chunk_context}"
            ),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_prompt_preserves_language_focus_and_chunk_ids() {
        let messages = summary_messages(
            Some("Agent Paper"),
            "Chinese",
            Some("limitations"),
            "[chunk-1] evidence",
        );
        let rendered = messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("Chinese"));
        assert!(rendered.contains("limitations"));
        assert!(rendered.contains("chunk-1"));
    }
}
