use std::sync::Arc;

use anyhow::{anyhow, Result};
use teloxide::prelude::*;

mod database_interactions;
mod process;
mod telegram_interactions;
mod translations;

pub async fn start(config_file_name: String) -> Result<()> {
    let config_file_string = std::fs::read_to_string(config_file_name)?;
    let json: serde_json::Value = serde_json::from_str(&config_file_string)?;
    let database_path = json
        .get("database_path")
        .ok_or(anyhow!("No database_path in json file!"))?
        .as_str()
        .ok_or(anyhow!("Error getting database_path!"))?;
    let pool = database_interactions::pool(database_path.to_string()).await?;
    database_interactions::check_cards_table(&pool).await?;
    let bot = Bot::new(
        json.get("bot_token")
        .ok_or(anyhow!("No bot_token in json file!"))?
        .as_str()
        .ok_or(anyhow!("Error getting bot_token!"))?
    );

    let translation = translations::translation()?;

    let handler = dptree::entry()
        .branch(Update::filter_message()
        .endpoint(telegram_interactions::message_handler))
        .branch(Update::filter_callback_query()
        .endpoint(telegram_interactions::callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, Arc::new(translation)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}