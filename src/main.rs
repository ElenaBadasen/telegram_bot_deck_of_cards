use anyhow::{anyhow, Result};
use std::env;
use telegram_bot_deck_of_cards::start;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, bot!");
    let subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let mut args = env::args();
    let config_file_name = match args.nth(1) {
        Some(arg) => arg,
        None => return Err(anyhow!("Didn't get config path param")),
    };

    start(config_file_name).await?;
    Ok(())
}
