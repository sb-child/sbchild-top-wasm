use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use chrono_tz::Tz;
use core::fmt::{self, Display, Formatter};
use flexstr::LocalStr;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

/// mini-tools API Key
///
/// - NOTE: This can't be shared between threads
#[derive(Clone)]
pub struct ApiKey {
    uid: LocalStr,
    key: LocalStr,
}

impl ApiKey {
    pub fn new(user_id: String, api_key: String) -> Self {
        Self {
            uid: user_id.into(),
            key: format!("Bearer {api_key}").into(),
        }
    }

    pub fn get_user_id_ref(&self) -> &str {
        self.uid.as_ref_type()
    }
    pub fn get_api_key_ref(&self) -> &str {
        self.key.as_ref_type()
    }
}

#[derive(Deserialize)]
pub enum KeepDuration {
    #[serde(rename = "1-day")]
    Keep1Day,
    #[serde(rename = "7-day")]
    Keep7Days,
    #[serde(rename = "30-day")]
    Keep30Days,
    #[serde(rename = "permanent")]
    Permanent,
}

impl Into<HeaderValue> for KeepDuration {
    fn into(self) -> HeaderValue {
        let r = HeaderValue::from_str(&self.to_string()).expect("Invalid KeepDuration header.");
        r
    }
}

impl Display for KeepDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            KeepDuration::Keep1Day => write!(f, "1-day"),
            KeepDuration::Keep7Days => write!(f, "7-day"),
            KeepDuration::Keep30Days => write!(f, "30-day"),
            KeepDuration::Permanent => write!(f, "permanent"),
        }
    }
}

#[derive(Deserialize)]
pub enum ImageMime {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

impl Display for ImageMime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ImageMime::Jpeg => write!(f, "image/jpeg"),
            ImageMime::Png => write!(f, "image/png"),
            ImageMime::Gif => write!(f, "image/gif"),
            ImageMime::Webp => write!(f, "image/webp"),
        }
    }
}

#[derive(Deserialize)]
pub struct UploadResp {
    success: bool,
    partial: bool,
    uploaded: Vec<UploadedItem>,
    failed: Vec<FailedItem>,
    usage: UsageObject,
}

#[derive(Deserialize)]
pub struct UploadedItem {
    success: bool,
    // example: 7-day/example.png
    key: LocalStr,
    // example: https://pub.mini-tools.uk/7-day/example.png
    url: LocalStr,
    duration: KeepDuration,
    // example: 2026-08-05T12:00:00.000Z
    // (UTC time)
    expires_at: DateTime<Utc>,
    size: u32,
    mime: ImageMime,
    // normal, 别的我试不出来
    risk: LocalStr,
}

#[derive(Deserialize)]
pub struct FailedItem {
    // example: 0
    index: u32,
    // example: aaa.png
    file_name: LocalStr,
    // example: 文件内容与图片格式不匹配（扩展名可能与真实格式不符）
    error: LocalStr,
    // example: INVALID_IMAGE_CONTENT
    code: LocalStr,
}

#[derive(Deserialize)]
pub struct UsageObject {
    // example: 23d2e827-d22a-48ea-8a74-7264a0d1f740
    user_id: LocalStr,
    // example: temporary_100
    plan_type: LocalStr,
    // example: 2026-10-11
    usage_date: LocalStr,
    temporary: TemporaryUsageObject,
    permanent: PermanentUsageObject,
}

#[derive(Deserialize)]
pub struct TemporaryUsageObject {
    enabled: bool,
    // example: 100
    limit: u32,
    // example: 1
    used: u32,
    // example: 99
    remaining: u32,
    // example: 2026-10-11T16:00:00.000Z
    // (UTC time)
    reset_at: DateTime<Utc>,
    // example: Asia/Shanghai
    // (不知道有什么用)
    timezone: Tz,
}

#[derive(Deserialize)]
pub struct PermanentUsageObject {
    enabled: bool,
    // example: 100
    limit: u32,
    // example: 1
    used: u32,
    // example: 99
    remaining: u32,
}
