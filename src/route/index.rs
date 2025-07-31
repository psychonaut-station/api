use api_derive::get;

#[get("/")]
pub(super) fn index() -> &'static str {
    "openapi docs here"
}
