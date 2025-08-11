use poem_openapi::{OpenApi, param::Path, payload::PlainText};

pub struct Endpoint;

#[OpenApi]
impl Endpoint {
    #[oai(path = "/player/:ckey", method = "get")]
    async fn player(&self, ckey: Path<String>) -> PlainText<String> {
        PlainText(format!("Player info for: {}", ckey.0))
    }
}
