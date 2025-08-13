use tokio::process::Command;

use super::Error;

pub async fn is_member(ckey: &str) -> Result<bool, Error> {
    // let response = REQWEST_CLIENT
    //     .get(format!(
    //         "https://secure.byond.com/members/{ckey}?format=text"
    //     ))
    //     .send()
    //     .await?;

    // if let Some(content_type) = response.headers().get("content-type") {
    //     if let Ok(content_type) = content_type.to_str() {
    //         return Ok(content_type.split(';').next() == Some("text/plain"));
    //     }
    // }

    let output = Command::new("/member.py").arg(ckey).output().await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();

        if stdout == "true" {
            return Ok(true);
        } else if stdout == "false" {
            return Ok(false);
        } else {
            return Err(Error::MemberCheckFailed(ckey.to_string()));
        }
    }

    Err(Error::MemberCheckFailed(ckey.to_string()))
}
