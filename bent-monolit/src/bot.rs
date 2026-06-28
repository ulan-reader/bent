use reqwest::Client;
use serde::Deserialize;
use teloxide::{
    prelude::*,
    types::{KeyboardButton, KeyboardMarkup},
};

#[derive(Debug, Deserialize)]
struct GenerateTokenResponse {
    url: String,
}

/// Запускает Telegram-бота. Вызывается из main() параллельно с HTTP-сервером.
///
/// ВАЖНО: бот дёргает API по адресу из TOKEN_API_URL (или localhost:8080
/// по умолчанию) — то есть даже находясь в одном процессе с axum-сервером,
/// общаются они через HTTP, не напрямую через функции. Это специально
/// оставлено так: при будущем разделении на отдельные бинари тебе не
/// придётся менять код бота вообще, просто вынесешь bot.rs в свой
/// Cargo.toml/main.rs как было раньше.
pub async fn run() {
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            match text {
                "/start" => {
                    let keyboard =
                        KeyboardMarkup::new(vec![vec![KeyboardButton::new("Обращение")]])
                            .resize_keyboard();
                    bot.send_message(msg.chat.id, "Выберите действие")
                        .reply_markup(keyboard)
                        .await?;
                }
                "Обращение" => {
                    match generate_link(&msg).await {
                        Ok(url) => {
                            bot.send_message(
                                msg.chat.id,
                                format!("Для создания обращения перейдите по ссылке:\n{url}"),
                            )
                            .await?;
                        }
                        Err(e) => {
                            log::error!("{e}");
                            bot.send_message(msg.chat.id, "Не удалось получить ссылку.")
                                .await?;
                        }
                    };
                }
                _ => {}
            }
        }
        respond(())
    })
    .await;
}

async fn generate_link(msg: &Message) -> Result<String, reqwest::Error> {
    let api_url =
        std::env::var("TOKEN_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = Client::new();
    let body = serde_json::json!({
        "telegram_user_id": msg.chat.id.0
    });

    let response = client
        .post(format!("{api_url}/api/token/generate"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<GenerateTokenResponse>()
        .await?;

    Ok(response.url)
}
