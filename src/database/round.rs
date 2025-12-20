use chrono::NaiveDateTime;
use rocket::futures::StreamExt as _;
use serde::Serialize;

use serde_json::Value;
use sqlx::{Executor as _, MySqlPool, Row as _};

use crate::{config::Config, database::*};

use super::error::Error;

#[derive(Debug, Serialize)]
pub struct RoundData {
    pub round_id: i32,
    pub server_ip: u32,
    pub server_port: u16,
    pub map_name: Option<String>,
    pub station_name: Option<String>,
    pub commit_hash: Option<String>,
    pub population: Vec<(String, i64)>,
    pub shuttle_name: Option<String>,
    pub nukedisk: Option<Value>,
    pub antagonists: Vec<Value>,
    pub dynamic_tier: Option<i32>,
    pub storyteller: Option<String>,
    #[serde(with = "crate::serde::datetime")]
    pub initialize_datetime: NaiveDateTime,
    #[serde(with = "crate::serde::opt_datetime")]
    pub start_datetime: Option<NaiveDateTime>,
    #[serde(with = "crate::serde::opt_datetime")]
    pub shutdown_datetime: Option<NaiveDateTime>,
    #[serde(with = "crate::serde::opt_datetime")]
    pub end_datetime: Option<NaiveDateTime>,
}

pub async fn get_round(
    round_id: i32,
    config: &Config,
    pool: &MySqlPool,
) -> Result<RoundData, Error> {
    let current_round_id = get_round_id(config).await?;

    if let Some(current_round_id) = current_round_id {
        if current_round_id == round_id {
            return Err(Error::RoundNotFound);
        }
    }

    let mut connection = pool.acquire().await?;

    let nukedisk_feedback = get_feedback(
        "associative",
        "roundend_nukedisk",
        round_id,
        &mut connection,
    )
    .await?;
    let dynamic_tier_feedback =
        get_feedback("associative", "dynamic_tier", round_id, &mut connection).await?;
    let antagonists_feedback =
        get_feedback("associative", "antagonists", round_id, &mut connection).await?;
    let storyteller_feedback =
        get_feedback("associative", "storyteller", round_id, &mut connection).await?;

    let nukedisk: Option<Value> = nukedisk_feedback
        .and_then(|fb| fb.json.get("data").cloned())
        .and_then(|data| data.get("1").cloned());
    let dynamic_tier: Option<i32> = dynamic_tier_feedback
        .and_then(|fb| fb.json.get("data").cloned())
        .and_then(|data| data.get("1").cloned())
        .and_then(|data| data.get("tier").cloned())
        .and_then(|v| v.as_str()?.parse::<i32>().ok());
    let storyteller: Option<String> = storyteller_feedback
        .and_then(|fb| fb.json.get("data").cloned())
        .and_then(|data| data.get("1").cloned())
        .and_then(|data| data.get("name").cloned())
        .and_then(|value| value.as_str().map(String::from));
    let antagonists_objects: Option<Value> =
        antagonists_feedback.and_then(|fb| fb.json.get("data").cloned());
    let antagonists: Vec<Value> = match antagonists_objects {
        Some(Value::Object(ref map)) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort_by_key(|k| k.parse::<usize>().unwrap());
            keys.iter().map(|k| map[k.as_str()].clone()).collect()
        }
        _ => Vec::new(),
    };

    let query = sqlx::query(
        "SELECT id, server_ip, server_port, map_name, station_name, commit_hash, shuttle_name, initialize_datetime, start_datetime, shutdown_datetime, end_datetime FROM round WHERE id = ? ORDER BY id DESC",
    )
    .bind(round_id);

    let Ok(row) = connection.fetch_one(query).await else {
        return Err(Error::RoundNotFound);
    };

    let population =
        get_population(round_id, Some(row.try_get("initialize_datetime")?), pool).await?;

    let round = RoundData {
        round_id: row.try_get("id")?,
        server_ip: row.try_get("server_ip")?,
        server_port: row.try_get("server_port")?,
        map_name: row.try_get("map_name")?,
        station_name: row.try_get("station_name")?,
        commit_hash: row.try_get("commit_hash")?,
        shuttle_name: row.try_get("shuttle_name")?,
        initialize_datetime: row.try_get("initialize_datetime")?,
        start_datetime: row.try_get("start_datetime")?,
        shutdown_datetime: row.try_get("shutdown_datetime")?,
        end_datetime: row.try_get("end_datetime")?,
        population,
        nukedisk,
        antagonists,
        dynamic_tier,
        storyteller,
    };

    connection.close().await?;

    Ok(round)
}

pub async fn get_population(
    round_id: i32,
    initialize_date: Option<NaiveDateTime>,
    pool: &MySqlPool,
) -> Result<Vec<(String, i64)>, Error> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT time, playercount FROM legacy_population WHERE round_id = ? ORDER BY time ASC;",
    )
    .bind(round_id);

    let mut population = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let datetime: NaiveDateTime = row.try_get("time")?;

            let time_str = if let Some(start) = initialize_date {
                let duration = datetime - start;

                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                let seconds = duration.num_seconds() % 60;
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            } else {
                datetime.format("%H:%M:%S").to_string()
            };

            population.push((time_str, row.try_get("playercount")?));
        }
    }

    connection.close().await?;

    Ok(population)
}

pub async fn get_rounds(
    fetch_size: Option<i32>,
    page: Option<i32>,
    autocomplete_round_id: Option<i32>,
    config: &Config,
    pool: &MySqlPool,
) -> Result<(Vec<RoundData>, i64), Error> {
    let round_id = get_round_id(config).await?;

    let fetch_size = fetch_size.unwrap_or(20);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT COUNT(*) FROM round WHERE map_name IS NOT NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND id < ?");
    }

    if autocomplete_round_id.is_some() {
        sql.push_str(" AND id LIKE CONCAT(?, '%')");
    }

    let mut query = sqlx::query_scalar(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }
    if let Some(autocomplete_round_id) = autocomplete_round_id {
        query = query.bind(autocomplete_round_id);
    }
    let total_count = query.fetch_one(&mut *connection).await?;

    let mut sql = "SELECT id, server_ip, server_port, map_name, station_name, commit_hash, game_mode, game_mode_result, end_state, shuttle_name, initialize_datetime, start_datetime, shutdown_datetime, end_datetime FROM round WHERE map_name IS NOT NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND id < ?");
    }
    if autocomplete_round_id.is_some() {
        sql.push_str(" AND id LIKE CONCAT(?, '%')");
    }

    sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }
    if let Some(autocomplete_round_id) = autocomplete_round_id {
        query = query.bind(autocomplete_round_id);
    }

    query = query.bind(fetch_size).bind(offset);

    let mut rounds = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let row = row?;

            let round = RoundData {
                round_id: row.try_get("id")?,
                server_ip: row.try_get("server_ip")?,
                server_port: row.try_get("server_port")?,
                map_name: row.try_get("map_name")?,
                station_name: row.try_get("station_name")?,
                commit_hash: row.try_get("commit_hash")?,
                shuttle_name: row.try_get("shuttle_name")?,
                initialize_datetime: row.try_get("initialize_datetime")?,
                start_datetime: row.try_get("start_datetime")?,
                shutdown_datetime: row.try_get("shutdown_datetime")?,
                end_datetime: row.try_get("end_datetime")?,
                population: Vec::new(),
                antagonists: Vec::new(),
                nukedisk: None,
                dynamic_tier: None,
                storyteller: None,
            };

            rounds.push(round);
        }
    }

    connection.close().await?;

    Ok((rounds, total_count))
}
