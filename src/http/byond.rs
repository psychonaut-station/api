use super::{Error, REQWEST_CLIENT};

pub async fn is_member(ckey: &str) -> Result<bool, Error> {
    let response = REQWEST_CLIENT
        .get(format!(
            "http://selenium-proxy:8000/?url=https://secure.byond.com/members/{ckey}?format=text"
        ))
        .send()
        .await?;

    if let Some(content_length) = response.headers().get("content-length") {
        if let Ok(Ok(content_length)) = content_length.to_str().map(|s| s.parse::<u32>()) {
            return Ok(content_length < 1000);
        }
    }

    Ok(false)
}
