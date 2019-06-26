use crate::config::*;
use failure::Error;
use reqwest::Client;

#[derive(Serialize, Deserialize)]
struct PostMessageResponse {
    id: i32,
    msg: String,
    result: String,
}

#[derive(Debug, Fail)]
enum ZulipError {
    #[fail(display = "Failed to post message: {}", msg)]
    PostError { msg: String },
}

pub fn post_message(stream: &str, topic: &str, message: &str) -> Result<i32, Error> {
    let client = Client::new();
    let content = vec![
        format!("type=stream"),
        format!("to={}", stream),
        format!("subject={}", topic),
        format!("content={}", message),
    ]
    .join("&");

    let response = client
        .post(format!("{}/api/v1/messages", &ZULIP_URL).as_str())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .basic_auth(&CONFIG.zulip_user, Some(&CONFIG.zulip_token))
        .body(content)
        .send()?
        .json::<PostMessageResponse>()?;

    if response.result == "success" {
        Ok(response.id)
    } else {
        Err(Error::from(ZulipError::PostError { msg: response.msg }))
    }
}
