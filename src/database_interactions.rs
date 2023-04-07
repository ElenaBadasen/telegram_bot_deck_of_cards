use anyhow::Result;
use rand::seq::IteratorRandom;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{fs::File, time::SystemTime};
use time::{OffsetDateTime};

use crate::process::{Card, CardData, self};

pub async fn pool(database_path: String) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_path)
        .await?;
    Ok(pool)
}

pub async fn get_active_bot_id(pool: &PgPool) -> Result<i32> {
    let query_result = sqlx::query_scalar!(
        "
        SELECT id FROM bots WHERE active=true
    "
    )
    .fetch_one(pool)
    .await?;
    Ok(query_result)
}

pub async fn get_subscriber_id(chat_id: String, pool: &PgPool) -> Result<Option<i32>> {
    let query_result = sqlx::query_scalar!(
        "
        SELECT id FROM subscribers 
        WHERE chat_id=$1;
    ",
        chat_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(query_result)
}

pub async fn get_language(subscriber_id: i32, pool: &PgPool) -> Result<process::Language> {
    let query_result = sqlx::query_scalar!( 
        r#"
        SELECT language AS "language: process::Language"
        FROM chats_states 
        WHERE subscriber_id=$1;
    "#,
        subscriber_id
    )
    .fetch_one(pool)
    .await?;
    Ok(query_result)
}

pub async fn get_descriptions_format(subscriber_id: i32, pool: &PgPool) -> Result<i32> {
    let query_result = sqlx::query_scalar!(
        "
        SELECT descriptions_format FROM chats_states 
        WHERE subscriber_id=$1;
    ",
        subscriber_id
    )
    .fetch_one(pool)
    .await?;
    Ok(query_result)
}

pub async fn create_subscriber(bot_id: i32, chat_id: String, pool: &PgPool) -> Result<i32> {
    let system_time = SystemTime::now();
    let t: OffsetDateTime = system_time.into();
    let mut tx = pool.begin().await?;
    let subscriber_id = sqlx::query_scalar!(
        "
            INSERT INTO subscribers (chat_id, created_at) 
            values ($1, $2)
            RETURNING id;
        ",
        chat_id,
        t
    )
    .fetch_one(&mut tx)
    .await?;
    sqlx::query!(
        "
            INSERT INTO bots_subscribers (bot_id, subscriber_id) 
            values ($1, $2);
        ",
        bot_id,
        subscriber_id
    )
    .execute(&mut tx)
    .await?;
    sqlx::query!(
        "
            INSERT INTO chats_states 
            (bot_id, subscriber_id, drawn_cards, language, descriptions_format) 
            values ($1, $2, $3, 'en', $4);
        ",
        bot_id,
        subscriber_id,
        &[],
        process::FULL_DESCRIPTIONS
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(subscriber_id)
}

pub async fn set_description(value: i32, subscriber_id: i32, pool: &PgPool) -> Result<()> {
    let active_bot_id = get_active_bot_id(pool).await?;
    sqlx::query!(
        "
            UPDATE chats_states SET descriptions_format=$1 
            WHERE subscriber_id=$2 AND bot_id=$3;
        ",
        value,
        subscriber_id,
        active_bot_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_language(language: process::Language, subscriber_id: i32, pool: &PgPool) -> Result<()> {
    let active_bot_id = get_active_bot_id(pool).await?;
    sqlx::query!(
        "
            UPDATE chats_states SET language=$1 
            WHERE subscriber_id=$2 AND bot_id=$3;
        ",
        language as process::Language,
        subscriber_id,
        active_bot_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_file_id(filename: String, id: String, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
            UPDATE cards
            SET telegram_file_id_en = $1
            WHERE filename=$2;
        ",
        id,
        filename
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn shuffle_cards_back(subscriber_id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        UPDATE chats_states
        SET drawn_cards = $1
        WHERE subscriber_id = $2;
    ",
        &[],
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn random_card_file_name(
    language: process::Language,
    descriptions: i32,
    subscriber_id: i32,
    pool: &PgPool,
) -> Result<Option<CardData>> {
    let cards: Vec<Card> = sqlx::query_as!(
        Card,
        "
            SELECT * FROM cards;
        "
    )
    .fetch_all(pool)
    .await?;

    let mut skip_cards_ids: Vec<i32> = sqlx::query!(
        "
            SELECT drawn_cards FROM chats_states WHERE subscriber_id = $1;
        ",
        subscriber_id
    )
    .fetch_one(pool)
    .await?
    .drawn_cards;

    let chosen_card = cards.iter().filter(|c| !skip_cards_ids.contains(&c.id)).choose(&mut rand::thread_rng());
    if let Some(chosen_card) = chosen_card {
        //save drawn card to database
        skip_cards_ids.push(chosen_card.id);
        sqlx::query!(
            "
                UPDATE chats_states 
                SET drawn_cards=$1
                WHERE subscriber_id = $2;
            ",
            &skip_cards_ids,
            subscriber_id
        )
        .execute(pool)
        .await?;

        let file_id = chosen_card.get_file_id(language);
        let message_text = match descriptions {
            process::FULL_DESCRIPTIONS => {
                let name = chosen_card.get_name(language);
                let description = chosen_card.get_description(language);
                format!("{}\n{}", name, description)
            }
            process::NAMES_ONLY => {
                let name = chosen_card.get_name(language);
                name
            },
            process::NO_DESCRIPTIONS => "".to_string(),
            _ => {
                panic!("Descriptions format not supported!");
            } 
        };

        Ok(Some(CardData {
            filename: chosen_card.filename.clone(),
            message_text,
            file_id,
        }))
    } else {
        Ok(None)
    }
}

pub async fn check_cards_table(pool: &PgPool) -> Result<()> {
    let count = sqlx::query_scalar!(
        "
            SELECT COUNT(*) FROM cards;
        "
    )
    .fetch_one(pool)
    .await?;
    if count == Some(0) {
        //fill the table from csv file
        let file = File::open("pictures/data.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        let mut tx = pool.begin().await?;
        for result in rdr.records() {
            let record = result?;
            sqlx::query!(
                "
                INSERT INTO cards 
                (filename, name_en, description_en, name_ru, description_ru)
                VALUES ($1, $2, $3, $4, $5);
            ",
                record[0].to_string(),
                record[1].to_string(),
                record[2].to_string(),
                record[3].to_string(),
                record[4].to_string()
            )
            .execute(&mut tx)
            .await?;
        }
        tx.commit().await?;
    }
    Ok(())
}