use poem_openapi::{OpenApi, payload::PlainText};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    #[oai(path = "/recent-test-merges.json", method = "get")]
    async fn recent_test_merges(&self) -> PlainText<String> {
        PlainText("Recent Test Merges".to_string())
    }
}
