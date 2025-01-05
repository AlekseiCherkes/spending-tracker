use chrono::Utc;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, UpdateKind};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Spending Tracker bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot.clone(),
        dptree::entry()
            .branch(Update::filter_message().endpoint(handle_message))
            .branch(Update::filter_callback_query().endpoint(handle_callback_query)),
    )
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn handle_message(
    bot: Bot,
    message: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(text) = message.text() {
        if let Ok(amount) = text.parse::<f64>() {
            let timestamp = Utc::now().to_rfc3339();
            let user = message
                .from()
                .and_then(|u| u.username.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let account = "default_account".to_string();
            let currency = "USD".to_string();

            // Inline keyboard for editing properties
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback(
                    format!("Amount: {}", amount),
                    "edit_amount",
                )],
                vec![InlineKeyboardButton::callback(
                    format!("Timestamp: {}", timestamp),
                    "edit_timestamp",
                )],
                vec![InlineKeyboardButton::callback(
                    format!("Account: {}", account),
                    "edit_account",
                )],
                vec![InlineKeyboardButton::callback(
                    format!("Currency: {}", currency),
                    "edit_currency",
                )],
                vec![InlineKeyboardButton::callback("Commit", "commit")],
            ]);

            // Send message with inline keyboard
            bot.send_message(
                message.chat.id,
                "Expense created! Edit properties if needed:",
            )
            .reply_markup(keyboard)
            .await?;
        } else {
            bot.send_message(message.chat.id, "Please send a valid amount.")
                .await?;
        }
    }
    Ok(())
}

async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(data) = q.data.clone() {
        match data.as_str() {
            "edit_amount" => {
                bot.send_message(q.from.id, "Send the new amount:").await?;
                // Implement logic to handle the next message as the new amount
            }
            "edit_timestamp" => {
                bot.send_message(q.from.id, "Send the new timestamp:")
                    .await?;
                // Implement logic to handle the next message as the new timestamp
            }
            "edit_account" => {
                bot.send_message(q.from.id, "Send the new account:").await?;
                // Implement logic to handle the next message as the new account
            }
            "edit_currency" => {
                bot.send_message(q.from.id, "Send the new currency:")
                    .await?;
                // Implement logic to handle the next message as the new currency
            }
            "commit" => {
                // Save to DB (implement SQLite logic here)
                bot.send_message(q.from.id, "Expense saved successfully!")
                    .await?;
            }
            _ => {
                bot.send_message(q.from.id, "Unknown action.").await?;
            }
        }

        bot.answer_callback_query(&q.id.clone()).await?;

        // Edit text of the message to which the buttons were attached
        if let Some(message) = q.regular_message() {
            bot.edit_message_text(message.chat.id, message.id, String::from("new text"))
                .await?;
        }
    }
    Ok(())
}
