use crate::lightbox::{
    errors::{BaseError, HttpSnafu, JsonSnafu},
    model::{self, ApiKey, KeepDuration, UploadResp},
};
use reqwest::{multipart, StatusCode};
use snafu::{futures::TryFutureExt as _, ResultExt as _};

async fn upload_single_inner(
    cli: reqwest::Client,
    cred: &ApiKey,
    dur: KeepDuration,
    image: multipart::Part,
) -> Result<(StatusCode, UploadResp), BaseError> {
    // let file_data: Vec<u8> = vec![];
    // let file_part = multipart::Part::bytes(file_data)
    //     .file_name("photo.webm")
    //     .mime_str("image/webm")?;
    let form = multipart::Form::new().part("file", image);
    let res = cli
        .post("https://api.mini-tools.uk/v1/upload")
        .header("X-API-User-ID", cred.get_user_id_ref())
        .header("Authorization", cred.get_api_key_ref())
        // .header("Idempotency-Key", "")
        .header("duration", dur)
        .multipart(form)
        .send()
        .context(HttpSnafu)
        .await?;
    let status = res.status();
    let resp = res.bytes().context(HttpSnafu).await?;
    let resp_json: UploadResp = serde_json::from_slice(&resp).context(JsonSnafu)?;
    Ok((status, resp_json))
}

pub enum UploadSingleResult {}

pub async fn upload_single() {}
