use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

/// HTTP client for communicating with an Antfly server.
pub struct AntflyClient {
    base_url: Url,
    http: Client,
}

#[derive(Debug, Serialize)]
pub struct SearchRequest<'a> {
    pub table: &'a str,
    pub query_string: &'a str,
    pub fields: Vec<&'a str>,
    pub limit: Option<i64>,
    pub end_user_search: &'a str,
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub score: f64,
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

#[derive(Debug)]
pub enum ClientError {
    InvalidUrl(String, url::ParseError),
    Request(reqwest::Error),
    ResponseFormat(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::InvalidUrl(url, err) => write!(f, "invalid url '{}': {}", url, err),
            ClientError::Request(err) => write!(f, "request failed: {}", err),
            ClientError::ResponseFormat(msg) => write!(f, "invalid response: {}", msg),
        }
    }
}

impl AntflyClient {
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        let base_url =
            Url::parse(base_url).map_err(|e| ClientError::InvalidUrl(base_url.to_string(), e))?;
        Ok(Self {
            base_url,
            http: Client::new(),
        })
    }

    /// Search an Antfly collection, returning raw JSON hits.
    pub fn search(&self, req: &SearchRequest) -> Result<Vec<serde_json::Value>, ClientError> {
        let url = self
            .base_url
            .join("query")
            .map_err(|e| ClientError::InvalidUrl("query".to_string(), e))?;

        let resp = self
            .http
            .post(url)
            .json(req)
            .send()
            .map_err(ClientError::Request)?
            .error_for_status()
            .map_err(ClientError::Request)?;

        let body: serde_json::Value = resp.json().map_err(ClientError::Request)?;

        match body {
            serde_json::Value::Array(rows) => Ok(rows),
            _ => Err(ClientError::ResponseFormat(
                "expected JSON array".to_string(),
            )),
        }
    }

    /// Notify Antfly to sync a document.
    pub fn sync_document(
        &self,
        collection: &str,
        doc_id: &str,
        doc: &serde_json::Value,
    ) -> Result<(), ClientError> {
        let path = format!("collections/{}/documents/{}", collection, doc_id);
        let url = self
            .base_url
            .join(&path)
            .map_err(|e| ClientError::InvalidUrl(path, e))?;

        self.http
            .put(url)
            .json(doc)
            .send()
            .map_err(ClientError::Request)?
            .error_for_status()
            .map_err(ClientError::Request)?;

        Ok(())
    }

    /// Notify Antfly to delete a document.
    pub fn delete_document(&self, collection: &str, doc_id: &str) -> Result<(), ClientError> {
        let path = format!("collections/{}/documents/{}", collection, doc_id);
        let url = self
            .base_url
            .join(&path)
            .map_err(|e| ClientError::InvalidUrl(path, e))?;

        self.http
            .delete(url)
            .send()
            .map_err(ClientError::Request)?
            .error_for_status()
            .map_err(ClientError::Request)?;

        Ok(())
    }
}
