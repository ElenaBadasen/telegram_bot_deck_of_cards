use std::sync::Arc;

use anyhow::{Error, Result};
use sqlx::postgres::PgPool;
use teloxide::{
    prelude::*,
    types::{Chat, Me, MediaKind, MessageId, MessageKind, MessageCommon},
    utils::command::BotCommands,
};

use crate::database_interactions;
use crate::process;
use crate::translations;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Get a list of actions.")]
    Start,
    #[command(description = "Get a list of actions.")]
    Help,
    #[command(description = "Show settings.")]
    Settings,
    #[command(description = "Main menu.")]
    MainMenu,
    #[command(description = "Get a description of project.")]
    About,
    #[command(description = "Send me a random card.")]
    Card,
    #[command(description = "Shuffle the drawn cards back into deck.")]
    Shuffle,
    #[command(description = "Choose language.")]
    Language,
    #[command(description = "Set English language.")]
    En,
    #[command(description = "Set Russian language.")]
    Ru,
    #[command(description = "Choose description settings.")]
    Description,
    #[command(description = "Set full description.")]
    FullDescription,
    #[command(description = "Set names only.")]
    NamesOnly,
    #[command(description = "No descriptions.")]
    NoDescription,
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
    pool: PgPool,
    translation: Arc<translations::Translation>,
) -> Result<()> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(command) => {
                process(command, msg.chat, msg.id, bot, None, &pool, &translation).await?;
            }
            Err(_) => {
                send_error_message("command_not_found", msg.chat, bot, &pool, &translation).await?;
            }
        }
    } else {
        send_error_message("text_expected", msg.chat, bot, &pool, &translation).await?;
    }
    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    pool: PgPool,
    translation: Arc<translations::Translation>,
) -> Result<()> {
    if let Some(request) = q.data.clone() {
        let command: Command = match request.as_str() {
            translations::ABOUT_EN | translations::ABOUT_RU => Command::About,
            translations::MAIN_MENU_EN | translations::MAIN_MENU_RU => Command::MainMenu,
            translations::CARD_EN | translations::CARD_RU => Command::Card,
            translations::SHUFFLE_EN | translations::SHUFFLE_RU => Command::Shuffle,
            translations::SETTINGS_EN | translations::SETTINGS_RU => Command::Settings,
            translations::LANGUAGE_EN | translations::LANGUAGE_RU => Command::Language,
            translations::DESCRIPTIONS_EN | translations::DESCRIPTIONS_RU => Command::Description,
            translations::EN_EN => Command::En,
            translations::RU_RU => Command::Ru,
            translations::FULL_DESCRIPTIONS_EN | translations::FULL_DESCRIPTIONS_RU => {
                Command::FullDescription
            }
            translations::NAMES_ONLY_EN | translations::NAMES_ONLY_RU => Command::NamesOnly,
            translations::NO_DESCRIPTIONS_EN | translations::NO_DESCRIPTIONS_RU => {
                Command::NoDescription
            }
            _other => {
                if let Some(Message { chat, .. }) = q.message.clone() {
                    send_error_message("command_not_found", chat, bot, &pool, &translation).await?;
                }
                return Ok(());
            }
        };

        if let Some(Message { id, chat, .. }) = q.message.clone() {
            process(command, chat, id, bot, Some(q), &pool, &translation).await?;
        }
    }
    Ok(())
}

async fn process(
    command: Command,
    chat: Chat,
    message_id: MessageId,
    bot: Bot,
    q: Option<CallbackQuery>,
    pool: &PgPool,
    translation: &translations::Translation,
) -> Result<()> {
    let action =
        process::process_message(command, chat.id.to_string(), pool, translation).await;
    match action {
        Ok(action_inner) => {
            let result =
                process_action(action_inner, chat.clone(), message_id, bot.clone(), q, pool).await;
            if let Err(e) = result {
                _ = log_error(chat, bot, e, pool, translation).await;
            }
        }
        Err(e) => {
            _ = log_error(chat, bot, e, pool, translation).await;
        }
    }
    Ok(())
}

async fn log_error(
    chat: Chat,
    bot: Bot,
    e: Error,
    pool: &PgPool,
    translation: &translations::Translation,
) -> Result<()> {
    let err_str = format!("Error: {e:?}");
    tracing::info!(err_str);
    send_error_message("unknown_error", chat, bot, pool, translation).await?;
    Ok(())
}

async fn send_error_message(message_key: &str, chat: Chat, bot: Bot, 
    pool: &PgPool, translation: &translations::Translation) -> Result<()> {
    let subscriber_id = database_interactions::get_subscriber_id(chat.id.to_string(), pool).await?;
    let language = if let Some(subscriber_id) = subscriber_id {
        database_interactions::get_language(subscriber_id, pool).await?
    } else {
        process::Language::En
    };
    _ = bot
        .send_message(
            chat.id,
            translation.get(message_key, language)?,
        )
        .await;
    Ok(())
}

pub async fn process_action(
    action: process::Action,
    chat: Chat,
    message_id: MessageId,
    bot: Bot,
    q: Option<CallbackQuery>,
    pool: &PgPool,
) -> Result<()> {
    if let Some(q) = q {
        bot.answer_callback_query(q.id).await?;
    }
    if action.delete_previous_message {
        //if deletion failed, then whatever, it doesn't work after 48 hours
        let _ = bot.delete_message(chat.id, message_id).await;
    } else if let Some(replacement_text) = action.replacement_text {
        let edit_result = bot
            .edit_message_text(chat.id, message_id, replacement_text.clone())
            .await;
        if edit_result.is_err() {
            //if editing message doesn't work (e.g., after 48 hours), 
            //then just send the text separately
            bot.send_message(chat.id, replacement_text).await?;
        }
    }
    if let Some((input_file, image_description)) = action.image_data {
        let result = bot.send_photo(chat.id, input_file).await?;
        if let Some(filename) = action.filename {
            let file_id = match result.kind {
                MessageKind::Common(MessageCommon{media_kind: MediaKind::Photo(p), ..}) => {
                     Some(p.photo.first().unwrap().file.id.clone())
                },
                _ => None,
            };
            if let Some(id) = file_id {
                database_interactions::set_file_id(filename, id, pool).await?;
            }
        }
        if !image_description.is_empty() {
            bot.send_message(chat.id, image_description).await?;
        }
    }
    bot.send_message(chat.id, action.new_message_text)
        .reply_markup(action.keyboard)
        .await?;

    Ok(())
}
