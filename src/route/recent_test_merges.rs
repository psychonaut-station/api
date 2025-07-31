use api_derive::get;

#[get("/recent-test-merges.json")]
pub(super) fn recent_test_merges() -> &'static str {
    "Recent Test Merges"
}
