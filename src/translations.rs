use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::process;

pub struct Translation {
    en: HashMap<String, String>,
    ru: HashMap<String, String>,
}

impl Translation {
    pub fn get(
        &self,
        key: &str,
        language: process::Language,
    ) -> Result<String> {
        let selected_translation = match language {
            process::Language::En => &self.en,
            process::Language::Ru => &self.ru,
        };
        Ok(selected_translation
            .get(key)
            .ok_or(anyhow!("Missing translation!"))?
            .to_string())
    }
}

pub fn translation() -> Result<Translation> {
    let file_string_ru = std::fs::read_to_string("pictures/ru/translation.yml")?;
    let file_string_en = std::fs::read_to_string("pictures/en/translation.yml")?;
    let result_ru: HashMap<String, String> = serde_yaml::from_str(&file_string_ru)?;
    let result_en: HashMap<String, String> = serde_yaml::from_str(&file_string_en)?;
    Ok(Translation{
        en: result_en,
        ru: result_ru,
    })
}



//consts for buttons' texts
pub const ABOUT_EN: &str = "About";
pub const ABOUT_RU: &str = "О проекте";
pub const MAIN_MENU_EN: &str = "Main menu";
pub const MAIN_MENU_RU: &str = "Главное меню";
pub const CARD_EN: &str = "Draw a card";
pub const CARD_RU: &str = "Вытянуть карту";
pub const SHUFFLE_EN: &str = "Shuffle drawn cards back";
pub const SHUFFLE_RU: &str = "Замешать вытянутые карты в колоду";
pub const SETTINGS_EN: &str = "Settings";
pub const SETTINGS_RU: &str = "Настройки";
pub const LANGUAGE_EN: &str = "Language";
pub const LANGUAGE_RU: &str = "Язык";
pub const DESCRIPTIONS_EN: &str = "Descriptions";
pub const DESCRIPTIONS_RU: &str = "Описания";
pub const EN_EN: &str = "English";
pub const RU_RU: &str = "Русский";
pub const FULL_DESCRIPTIONS_EN: &str = "Full descriptions";
pub const FULL_DESCRIPTIONS_RU: &str = "Полные описания";
pub const NAMES_ONLY_EN: &str = "Names only";
pub const NAMES_ONLY_RU: &str = "Только имена";
pub const NO_DESCRIPTIONS_EN: &str = "No descriptions";
pub const NO_DESCRIPTIONS_RU: &str = "Без описаний";
