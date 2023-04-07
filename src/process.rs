use anyhow::Result;
use sqlx::PgPool;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};

use crate::telegram_interactions::Command;
use crate::{database_interactions, translations};

fn make_keyboard(options: &[&str]) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for ops in options.chunks(1) {
        let row = ops
            .iter()
            .map(|&op| InlineKeyboardButton::callback(op.to_owned(), op.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

fn make_main_keyboard(language: Language) -> InlineKeyboardMarkup {
    let v = match language {
        Language::Ru => {
            vec![
                translations::CARD_RU,
                translations::SHUFFLE_RU,
                translations::SETTINGS_RU,
                translations::ABOUT_RU,
            ]
        }
        Language::En => {
            vec![
                translations::CARD_EN,
                translations::SHUFFLE_EN,
                translations::SETTINGS_EN,
                translations::ABOUT_EN,
            ]
        }
    };
    make_keyboard(&v)
}

fn make_settings_keyboard(language: Language) -> InlineKeyboardMarkup {
    let v = match language {
        Language::Ru => {
            vec![
                translations::LANGUAGE_RU,
                translations::DESCRIPTIONS_RU,
                translations::MAIN_MENU_RU,
            ]
        }
        Language::En => {
            vec![
                translations::LANGUAGE_EN,
                translations::DESCRIPTIONS_EN,
                translations::MAIN_MENU_EN,
            ]
        }
    };
    make_keyboard(&v)
}

fn make_languages_keyboard(language: Language) -> InlineKeyboardMarkup {
    let v = match language {
        Language::Ru => {
            vec![
                translations::EN_EN,
                translations::RU_RU,
                translations::SETTINGS_RU,
            ]
        }
        Language::En => {
            vec![
                translations::EN_EN,
                translations::RU_RU,
                translations::SETTINGS_EN,
            ]
        }
    };
    make_keyboard(&v)
}

fn make_descriptions_keyboard(language: Language) -> InlineKeyboardMarkup {
    let v = match language {
        Language::Ru => {
            vec![
                translations::FULL_DESCRIPTIONS_RU,
                translations::NAMES_ONLY_RU,
                translations::NO_DESCRIPTIONS_RU,
                translations::SETTINGS_RU,
            ]
        }
        Language::En => {
            vec![
                translations::FULL_DESCRIPTIONS_EN,
                translations::NAMES_ONLY_EN,
                translations::NO_DESCRIPTIONS_EN,
                translations::SETTINGS_EN,
            ]
        }
    };
    make_keyboard(&v)
}

pub struct Action {
    pub image_data: Option<(InputFile, String)>,
    pub delete_previous_message: bool,
    pub replacement_text: Option<String>,
    pub new_message_text: String,
    pub keyboard: InlineKeyboardMarkup,
    pub filename: Option<String>,
}

impl Action {
    fn new(new_message_text: String, keyboard: InlineKeyboardMarkup) -> Action {
        Action {
            image_data: None,
            delete_previous_message: false,
            replacement_text: None,
            new_message_text: new_message_text,
            keyboard: keyboard,
            filename: None,
        }
    }

    fn set_image_data(mut self, image_data: (InputFile, String)) -> Self {
        self.image_data = Some(image_data);
        self
    }

    fn set_delete_previous_message(mut self, delete_previous_message: bool) -> Self {
        self.delete_previous_message = delete_previous_message;
        self
    }

    fn set_replacement_text(mut self, replacement_text: String) -> Self {
        self.replacement_text = Some(replacement_text);
        self
    }

    fn set_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }
}

async fn check_subscriber(chat_id: String, pool: &PgPool) -> Result<i32> {
    let result = if let Some(id) = 
        database_interactions::get_subscriber_id(chat_id.clone(), pool)
        .await?
    {
        id
    } else {
        let bot_id = database_interactions::get_active_bot_id(pool).await?;
        database_interactions::create_subscriber(bot_id, chat_id, pool).await?
    };
    Ok(result)
}

pub async fn process_message(
    command: Command,
    chat_id: String,
    pool: &PgPool,
    translation: &translations::Translation,
) -> Result<Action> {
    let subscriber_id = check_subscriber(chat_id.clone(), pool).await?;
    let mut language = database_interactions::get_language(subscriber_id, pool).await?;
    let descriptions_format =
        database_interactions::get_descriptions_format(subscriber_id, pool).await?;
    let action: Action = match command {
        Command::Start => 
            Action::new(translation.get("start", language)?, 
                make_main_keyboard(language)),
        Command::Help => 
            Action::new(translation.get("help", language)?,
                make_main_keyboard(language))
            .set_delete_previous_message(true),
        Command::About => 
            Action::new(translation.get("choose_your_action", language)?,
                make_main_keyboard(language))
            .set_replacement_text(translation.get("description", language)?),
        Command::MainMenu => 
            Action::new(translation.get("choose_your_action", language)?,
                make_main_keyboard(language))
            .set_delete_previous_message(true),
        Command::Card => {
            if let Some(card_data) = database_interactions::random_card_file_name(
                language,
                descriptions_format,
                subscriber_id,
                pool,
            )
            .await?
            {
                let input_file = if let Some(file_id) = card_data.file_id {
                    InputFile::file_id(file_id)
                } else {
                    InputFile::file(format!("pictures/{}/{}", language.to_string(), card_data.filename))
                };

                Action::new(translation.get(
                    "choose_your_action",
                    language,
                )?, make_main_keyboard(language))
                .set_delete_previous_message(true)
                .set_image_data((input_file, card_data.message_text))
                .set_filename(card_data.filename)
            } else {
                Action::new(translation.get("no_cards_left", language)?,
                    make_main_keyboard(language))
                .set_delete_previous_message(true)
            }
        }
        Command::Shuffle => {
            database_interactions::shuffle_cards_back(subscriber_id, pool).await?;
            Action::new(translation.get("choose_your_action", language)?,
                make_main_keyboard(language))
            .set_replacement_text(translation.get(
                "cards_shuffled_back",
                language,
            )?)
        }
        Command::Settings => 
            Action::new(translation.get("settings", language)?,
                make_settings_keyboard(language))
            .set_delete_previous_message(true),
        Command::Language => 
            Action::new(translation.get("language_settings", language)?,
                make_languages_keyboard(language))
            .set_delete_previous_message(true),
        Command::Description => 
            Action::new(translation.get("descriptions_settings", language)?,
                make_descriptions_keyboard(language))
            .set_delete_previous_message(true),
        Command::En => {
            database_interactions::set_language(Language::En, subscriber_id, pool).await?;
            language = Language::En;
            Action::new(translation.get("language_settings", language)?,
                make_languages_keyboard(language))
            .set_replacement_text(translation.get(
                "language_set_to_english",
                language,
            )?)
        }
        Command::Ru => {
            database_interactions::set_language(Language::Ru, subscriber_id, pool).await?;
            language = Language::Ru;
            Action::new(translation.get("language_settings", language)?,
                make_languages_keyboard(language))
            .set_replacement_text(translation.get(
                "language_set_to_russian",
                language,
            )?)
        }
        Command::FullDescription => {
            database_interactions::set_description(FULL_DESCRIPTIONS, subscriber_id, pool).await?;
            Action::new(translation.get(
                "descriptions_settings",
                language,
            )?,
            make_descriptions_keyboard(language))
            .set_replacement_text(translation.get(
                "full_descriptions_will_be_shown",
                language,
            )?)
        }
        Command::NamesOnly => {
            database_interactions::set_description(NAMES_ONLY, subscriber_id, pool).await?;
            Action::new(translation.get(
                "descriptions_settings",
                language,
            )?,
            make_descriptions_keyboard(language))
            .set_replacement_text(translation.get(
                "names_only_will_be_shown",
                language,
            )?)
        }
        Command::NoDescription => {
            database_interactions::set_description(NO_DESCRIPTIONS, subscriber_id, pool).await?;
            Action::new(translation.get(
                "descriptions_settings",
                language,
            )?,
            make_descriptions_keyboard(language))
            .set_replacement_text(translation.get(
                "no_descriptions",
                language,
            )?)
        }
    };
    Ok(action)
}

#[derive(Debug, serde::Deserialize)]
pub struct Card {
    pub id: i32,
    pub filename: String,
    pub name_en: String,
    pub description_en: String,
    pub name_ru: String,
    pub description_ru: String,
    pub telegram_file_id_en: Option<String>,
    pub telegram_file_id_ru: Option<String>,
}

impl Card {
    pub fn get_name(&self, language: Language) -> String {
        match language {
            Language::En => {
                self.name_en.clone()
            }
            Language::Ru => {
                self.name_ru.clone()
            }
        }
    }

    pub fn get_description(&self, language: Language) -> String {
        match language {
            Language::En => {
                self.description_en.clone()
            }
            Language::Ru => {
                self.description_ru.clone()
            }
        }
    }

    pub fn get_file_id(&self, language: Language) -> Option<String> {
        match language {
            Language::En => {
                self.telegram_file_id_en.clone()
            }
            Language::Ru => {
                self.telegram_file_id_ru.clone()
            }
        }
    }
}

pub struct CardData {
    pub file_id: Option<String>,
    pub filename: String,
    pub message_text: String,
}

#[derive(sqlx::Type, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[sqlx(type_name = "language", rename_all = "lowercase")]
pub enum Language {
    En,
    Ru,
}

impl Language {
    fn to_string(&self) -> String {
        match self {
            Language::En => { "en".to_string() },
            Language::Ru => { "ru".to_string() },
        }
    }
}

pub const FULL_DESCRIPTIONS: i32 = 0;
pub const NAMES_ONLY: i32 = 1;
pub const NO_DESCRIPTIONS: i32 = 2;