use futures::TryStreamExt as _;
use poem_openapi::Object;
use sqlx::{FromRow, MySqlPool, Row as _, mysql::MySqlRow};

use crate::sqlxext::DateTime;

use super::Result;

#[derive(Object, Clone)]
pub struct TestMerge {
    /// The round ID of the test merge
    round_id: u32,
    /// The date and time when the test merge occurred
    /// in YYYY-MM-DD HH:MM:SS format
    datetime: String,
    /// The list of pull request numbers that were merged in this test merge
    test_merges: Vec<u32>,
}

impl FromRow<'_, MySqlRow> for TestMerge {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let test_merges = serde_json::from_str(row.try_get("test_merges")?).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "test_merges".into(),
                source: Box::new(e),
            }
        })?;
        Ok(TestMerge {
            round_id: row.try_get("round_id")?,
            datetime: row.try_get::<DateTime, _>("datetime")?.into(),
            test_merges,
        })
    }
}

pub async fn get_recent_test_merges(pool: &MySqlPool) -> Result<Vec<TestMerge>> {
    let query = sqlx::query_as(
        "SELECT round_id, datetime, JSON_ARRAYAGG(DISTINCT jt.number) AS test_merges FROM feedback JOIN JSON_TABLE(json, '$.data.*' COLUMNS(number INT PATH '$.number')) jt WHERE key_name = 'testmerged_prs' GROUP BY round_id, datetime ORDER BY round_id DESC LIMIT 200",
    );

    let mut test_merges = Vec::with_capacity(200);

    let mut stream = query.fetch(pool);

    while let Some(row) = stream.try_next().await? {
        test_merges.push(row);
    }

    Ok(test_merges)
}
