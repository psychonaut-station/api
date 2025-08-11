use poem::web::Data;
use poem_openapi::{
    ApiResponse, OpenApi,
    payload::{Json, PlainText},
};
use sqlx::MySqlPool;
use tracing::error;

use crate::{
    cache::Cache,
    database::{TestMerge, get_recent_test_merges},
};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    #[oai(path = "/recent-test-merges.json", method = "get")]
    async fn recent_test_merges(&self, pool: Data<&MySqlPool>, cache: Data<&Cache>) -> Response {
        if let Some(cached) = cache.get_recent_test_merges().await {
            return Response::Success(Json(cached));
        }

        let test_merges = match get_recent_test_merges(&pool).await {
            Ok(test_merges) => test_merges,
            Err(e) => {
                error!("Error fetching recent test merges: {e:?}");
                return Response::InternalError(e.into());
            }
        };

        cache.set_recent_test_merges(test_merges.clone()).await;

        Response::Success(Json(test_merges))
    }
}

#[derive(ApiResponse)]
enum Response {
    #[oai(status = 200)]
    Success(Json<Vec<TestMerge>>),
    #[oai(status = 500)]
    InternalError(PlainText<String>),
}
