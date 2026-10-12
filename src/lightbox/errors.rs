use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BaseError {
    #[snafu(display("HTTP Error: {source}"))]
    Http { source: reqwest::Error },

    #[snafu(display("JSON Error: {source}"))]
    Json { source: serde_json::Error },
}

pub enum UploadError {}
