use std::collections::HashMap;

use chrono::NaiveDateTime;
use rocket::futures::StreamExt as _;
use serde::Serialize;
use serde_json::Value;
use sqlx::{pool::PoolConnection, Executor as _, MySql, MySqlPool, Row as _};

use crate::{byond::get_server_status, config::Config};

use super::error::Error;

#[derive(Debug, Serialize)]
pub struct Feedback {
    pub round_id: Option<u32>,
    pub key_name: String,
    pub key_type: String,
    pub json: Value,
    #[serde(with = "crate::serde::datetime")]
    pub datetime: NaiveDateTime,
}

pub async fn get_feedback(
    key_name: &str,
    key_type: &str,
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<Vec<Feedback>, Error> {
    let mut sql = "SELECT datetime, round_id, key_name, key_type, json FROM feedback WHERE key_name = ? AND key_type = ?".to_string();

    if exclude_round.is_some() {
        sql.push_str(" AND round_id < ? AND round_id >= ?");
    }

    sql.push_str(" ORDER BY datetime DESC LIMIT ?");

    let mut query = sqlx::query(&sql).bind(key_name).bind(key_type);

    if let Some(round_id) = exclude_round {
        query = query.bind(round_id).bind(round_id - limit);
    }

    query = query.bind(limit);

    let mut feedbacks = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let feedback = row?;

            let feedback = Feedback {
                datetime: feedback.try_get("datetime")?,
                round_id: feedback.try_get("round_id")?,
                key_name: feedback.try_get("key_name")?,
                key_type: feedback.try_get("key_type")?,
                json: feedback.try_get("json")?,
            };

            feedbacks.push(feedback);
        }
    }

    Ok(feedbacks)
}

#[derive(Debug, Serialize)]
pub struct Death {
    pub name: String,
    pub job: String,
    pub pod: String,
    pub bruteloss: u16,
    pub fireloss: u16,
    pub oxyloss: u16,
    pub toxloss: u16,
    pub last_words: Option<String>,
    pub suicide: bool,
    pub round_id: Option<u32>,
    #[serde(with = "crate::serde::datetime")]
    pub tod: NaiveDateTime,
}

pub async fn get_deaths(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &Config,
    pool: &MySqlPool,
) -> Result<(Vec<Death>, i64), Error> {
    let round_id = get_round_id(config).await?;

    let fetch_size = fetch_size.unwrap_or(20);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT COUNT(*) FROM death".to_string();

    if round_id.is_some() {
        sql.push_str(" WHERE round_id < ?");
    }

    let mut query = sqlx::query_scalar(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    let total_count = query.fetch_one(&mut *connection).await?;

    let mut sql = "SELECT name, job, pod, bruteloss, fireloss, oxyloss, toxloss, last_words, suicide, round_id, tod FROM death".to_string();

    if round_id.is_some() {
        sql.push_str(" WHERE round_id < ?");
    }

    sql.push_str(" ORDER BY tod DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    query = query.bind(fetch_size).bind(offset);

    let mut deaths = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let death = row?;

            let death = Death {
                name: death.try_get("name")?,
                job: death.try_get("job")?,
                pod: death.try_get("pod")?,
                bruteloss: death.try_get("bruteloss")?,
                fireloss: death.try_get("fireloss")?,
                oxyloss: death.try_get("oxyloss")?,
                toxloss: death.try_get("toxloss")?,
                last_words: death.try_get("last_words")?,
                suicide: death.try_get("suicide")?,
                round_id: death.try_get("round_id")?,
                tod: death.try_get("tod")?,
            };

            deaths.push(death);
        }
    }

    connection.close().await?;

    Ok((deaths, total_count))
}

#[derive(Debug, Serialize)]
pub struct Crime {
    pub sender: String,
    pub recipient: String,
    pub crime: String,
    pub crime_desc: Option<String>,
    pub fine: Option<i32>,
    pub round_id: Option<u32>,
    #[serde(with = "crate::serde::datetime")]
    pub timestamp: NaiveDateTime,
}

pub async fn get_citations(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &Config,
    pool: &MySqlPool,
) -> Result<(Vec<Crime>, i64), Error> {
    let round_id = get_round_id(config).await?;

    let fetch_size = fetch_size.unwrap_or(20);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT COUNT(*) FROM citation WHERE fine IS NOT NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND round_id < ?");
    }

    let mut query = sqlx::query_scalar(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    let total_count = query.fetch_one(&mut *connection).await?;

    let mut sql =
        "SELECT round_id, sender_ic, recipient, crime, crime_desc, fine, timestamp FROM citation WHERE fine IS NOT NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND round_id < ?");
    }

    sql.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    query = query.bind(fetch_size).bind(offset);

    let mut citations = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let citation = row?;

            let citation = Crime {
                round_id: citation.try_get("round_id")?,
                sender: citation.try_get("sender_ic")?,
                recipient: citation.try_get("recipient")?,
                crime: citation.try_get("crime")?,
                crime_desc: citation.try_get("crime_desc")?,
                fine: citation.try_get("fine")?,
                timestamp: citation.try_get("timestamp")?,
            };

            citations.push(citation);
        }
    }

    connection.close().await?;

    Ok((citations, total_count))
}

pub async fn get_crimes(
    fetch_size: Option<i32>,
    page: Option<i32>,
    config: &Config,
    pool: &MySqlPool,
) -> Result<(Vec<Crime>, i64), Error> {
    let round_id = get_round_id(config).await?;

    let fetch_size = fetch_size.unwrap_or(20);
    let page = page.unwrap_or(1);
    let offset = (page - 1) * fetch_size;

    let mut connection = pool.acquire().await?;

    let mut sql = "SELECT COUNT(*) FROM citation WHERE fine IS NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND round_id < ?");
    }

    let mut query = sqlx::query_scalar(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    let total_count = query.fetch_one(&mut *connection).await?;

    let mut sql =
        "SELECT round_id, sender_ic, recipient, crime, crime_desc, fine, timestamp FROM citation WHERE fine IS NULL".to_string();

    if round_id.is_some() {
        sql.push_str(" AND round_id < ?");
    }

    sql.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = round_id {
        query = query.bind(round_id);
    }

    query = query.bind(fetch_size).bind(offset);

    let mut crimes = Vec::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let crime = row?;

            let crime = Crime {
                round_id: crime.try_get("round_id")?,
                sender: crime.try_get("sender_ic")?,
                recipient: crime.try_get("recipient")?,
                crime: crime.try_get("crime")?,
                crime_desc: crime.try_get("crime_desc")?,
                fine: crime.try_get("fine")?,
                timestamp: crime.try_get("timestamp")?,
            };

            crimes.push(crime);
        }
    }

    connection.close().await?;

    Ok((crimes, total_count))
}

pub async fn get_deaths_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, i64>, Error> {
    let mut sql = "SELECT round_id, COUNT(*) as deaths FROM death".to_string();

    if exclude_round.is_some() {
        sql.push_str(" WHERE round_id < ? AND round_id >= ?");
    }

    sql.push_str(" GROUP BY round_id ORDER BY round_id DESC LIMIT ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = exclude_round {
        query = query.bind(round_id).bind(round_id - limit);
    }

    query = query.bind(limit);

    let mut deaths = HashMap::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let death = row?;

            let round_id = death.try_get("round_id")?;
            let deaths_ = death.try_get("deaths")?;

            if let Some(round_id) = round_id {
                deaths.insert(round_id, deaths_);
            }
        }
    }

    Ok(deaths)
}

pub async fn get_citations_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, i64>, Error> {
    let mut sql =
        "SELECT round_id, COUNT(*) as citations FROM citation WHERE fine IS NOT NULL".to_string();

    if exclude_round.is_some() {
        sql.push_str(" AND round_id < ? AND round_id >= ?");
    }

    sql.push_str(" GROUP BY round_id ORDER BY round_id DESC LIMIT ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = exclude_round {
        query = query.bind(round_id).bind(round_id - limit);
    }

    query = query.bind(limit);

    let mut citations = HashMap::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let citation = row?;

            let round_id = citation.try_get("round_id")?;
            let citations_ = citation.try_get("citations")?;

            if let Some(round_id) = round_id {
                citations.insert(round_id, citations_);
            }
        }
    }

    Ok(citations)
}

pub async fn get_crimes_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, i64>, Error> {
    let mut sql =
        "SELECT round_id, COUNT(*) as crimes FROM citation WHERE fine IS NULL".to_string();

    if exclude_round.is_some() {
        sql.push_str(" AND round_id < ? AND round_id >= ?");
    }

    sql.push_str(" GROUP BY round_id ORDER BY round_id DESC LIMIT ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = exclude_round {
        query = query.bind(round_id).bind(round_id - limit);
    }

    query = query.bind(limit);

    let mut crimes = HashMap::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let crime = row?;

            let round_id = crime.try_get("round_id")?;
            let crimes_ = crime.try_get("crimes")?;

            if let Some(round_id) = round_id {
                crimes.insert(round_id, crimes_);
            }
        }
    }

    Ok(crimes)
}

pub async fn get_rounds_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, (i64, NaiveDateTime)>, Error> {
    let mut sql = "SELECT id, start_datetime, end_datetime FROM round".to_string();

    if exclude_round.is_some() {
        sql.push_str(" WHERE id < ? AND id >= ?");
    }

    sql.push_str(" ORDER BY id DESC LIMIT ?");

    let mut query = sqlx::query(&sql);

    if let Some(round_id) = exclude_round {
        query = query.bind(round_id).bind(round_id - limit);
    }

    query = query.bind(limit);

    let mut durations = HashMap::new();

    {
        let mut rows = connection.fetch(query);

        while let Some(row) = rows.next().await {
            let duration = row?;

            let round_id: i32 = duration.try_get("id")?;
            let start: Option<NaiveDateTime> = duration.try_get("start_datetime")?;
            let end: Option<NaiveDateTime> = duration.try_get("end_datetime")?;

            if let (Some(start), Some(end)) = (start, end) {
                let minutes = end.signed_duration_since(start).num_seconds() / 60;

                durations.insert(round_id as u32, (minutes, start));
            }
        }
    }

    Ok(durations)
}

pub async fn get_players_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, u32>, Error> {
    let feedback = get_feedback(
        "round_end_stats",
        "nested tally",
        limit,
        exclude_round,
        connection,
    )
    .await?;

    let mut players = HashMap::new();

    for feedback in &feedback {
        let players_ = feedback.json["data"]["players"]["total"]
            .as_u64()
            .unwrap_or(0);

        if let Some(round_id) = feedback.round_id {
            players.insert(round_id, players_ as u32);
        }
    }

    Ok(players)
}

pub async fn get_threat_overview(
    limit: i32,
    exclude_round: Option<i32>,
    connection: &mut PoolConnection<MySql>,
) -> Result<HashMap<u32, (i32, i32)>, Error> {
    let feedback = get_feedback(
        "dynamic_threat",
        "associative",
        limit,
        exclude_round,
        connection,
    )
    .await?;

    let mut threats = HashMap::new();

    for feedback in &feedback {
        let threat_level = feedback.json["data"]["1"]["threat_level"]
            .as_str()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let readied_players = feedback.json["data"]["1"]["player_count"]
            .as_str()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        if let Some(round_id) = feedback.round_id {
            threats.insert(round_id, (threat_level, readied_players));
        }
    }

    Ok(threats)
}

#[derive(Debug, Serialize)]
pub struct Overview {
    pub round_id: u32,
    pub duration: i64,
    #[serde(with = "crate::serde::datetime")]
    pub time: NaiveDateTime,
    pub citations: i64,
    pub crimes: i64,
    pub deaths: i64,
    pub players: u32,
    pub threat_level: i32,
    pub readied_players: i32,
}

pub async fn get_overview(
    limit: i32,
    config: &Config,
    pool: &MySqlPool,
) -> Result<Vec<Overview>, Error> {
    let mut connection = pool.acquire().await?;

    let exclude_round = get_round_id(config).await?;

    let rounds = get_rounds_overview(limit, exclude_round, &mut connection).await?;
    let deaths = get_deaths_overview(limit, exclude_round, &mut connection).await?;
    let citations = get_citations_overview(limit, exclude_round, &mut connection).await?;
    let crimes = get_crimes_overview(limit, exclude_round, &mut connection).await?;
    let players = get_players_overview(limit, exclude_round, &mut connection).await?;
    let threat_levels = get_threat_overview(limit, exclude_round, &mut connection).await?;

    connection.close().await?;

    let mut overview = Vec::new();

    for round in &rounds {
        let round_id = round.0;
        let (duration, time) = *round.1;

        let deaths = *deaths.get(round_id).unwrap_or(&0);
        let citations = *citations.get(round_id).unwrap_or(&0);
        let crimes = *crimes.get(round_id).unwrap_or(&0);
        let players = *players.get(round_id).unwrap_or(&0);
        let (threat_level, readied_players) = *threat_levels.get(round_id).unwrap_or(&(0, 0));

        let overview_ = Overview {
            round_id: *round_id,
            duration,
            time,
            deaths,
            citations,
            crimes,
            players,
            threat_level,
            readied_players,
        };

        overview.push(overview_);
    }

    Ok(overview)
}

pub async fn get_round_id(config: &Config) -> Result<Option<i32>, Error> {
    let status = get_server_status(config).await;
    let status = status.first();

    if let Some(status) = status {
        if let Some(round_id) = status.0.get("round_id") {
            return Ok(Some(round_id.as_u64().unwrap() as i32));
        }
    }

    Ok(None)
}
