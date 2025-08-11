use futures::StreamExt as _;
use poem_openapi::Object;
use sqlx::{Executor as _, MySqlPool, Row as _};

use crate::sqlxext::DateTime;

use super::Result;

#[derive(Object, Clone)]
pub struct TestMerge {
    round_id: u32,
    datetime: String,
    test_merges: Vec<u32>,
}

pub async fn get_recent_test_merges(pool: &MySqlPool) -> Result<Vec<TestMerge>> {
    let mut connection = pool.acquire().await?;

    let query = sqlx::query(
        "SELECT round_id, datetime, JSON_ARRAYAGG(DISTINCT jt.number) AS test_merges FROM feedback JOIN JSON_TABLE(json, '$.data.*' COLUMNS(number INT PATH '$.number')) jt WHERE key_name = 'testmerged_prs' GROUP BY round_id, datetime ORDER BY round_id DESC LIMIT 200",
    );

    let mut recent_test_merges = Vec::with_capacity(200);

    let mut rows = connection.fetch(query);

    while let Some(row) = rows.next().await {
        let row = row?;

        let test_merge = TestMerge {
            round_id: row.try_get("round_id")?,
            datetime: row.try_get::<DateTime, _>("datetime")?.into(),
            test_merges: serde_json::from_str(row.try_get("test_merges")?)?,
        };

        recent_test_merges.push(test_merge);
    }

    Ok(recent_test_merges)
}
