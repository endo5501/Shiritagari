use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AwClient {
    base_url: String,
    client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub id: String,
    #[serde(rename = "type")]
    pub bucket_type: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwEvent {
    pub id: Option<i64>,
    pub timestamp: String,
    pub duration: f64,
    pub data: HashMap<String, serde_json::Value>,
}

impl AwEvent {
    pub fn app(&self) -> Option<&str> {
        self.data.get("app").and_then(|v| v.as_str())
    }

    pub fn title(&self) -> Option<&str> {
        self.data.get("title").and_then(|v| v.as_str())
    }

    pub fn status(&self) -> Option<&str> {
        self.data.get("status").and_then(|v| v.as_str())
    }
}

#[derive(Debug)]
pub enum AwError {
    RequestFailed(String),
    ConnectionFailed(String),
}

impl std::fmt::Display for AwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwError::RequestFailed(msg) => write!(f, "AW request failed: {}", msg),
            AwError::ConnectionFailed(msg) => write!(f, "AW connection failed: {}", msg),
        }
    }
}

impl AwClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    pub fn default() -> Self {
        Self::new("http://localhost:5600")
    }

    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/0/buckets/", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }

    pub async fn get_buckets(&self) -> Result<HashMap<String, Bucket>, AwError> {
        let url = format!("{}/api/0/buckets/", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AwError::ConnectionFailed(e.to_string()))?;

        resp.json()
            .await
            .map_err(|e| AwError::RequestFailed(e.to_string()))
    }

    pub async fn get_events(
        &self,
        bucket_id: &str,
        start: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<AwEvent>, AwError> {
        let url = format!("{}/api/0/buckets/{}/events", self.base_url, bucket_id);

        let mut query_params: Vec<(&str, String)> = Vec::new();
        if let Some(start) = start {
            query_params.push(("start", start.to_string()));
        }
        if let Some(limit) = limit {
            query_params.push(("limit", limit.to_string()));
        }

        let resp = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| AwError::ConnectionFailed(e.to_string()))?;

        resp.json()
            .await
            .map_err(|e| AwError::RequestFailed(e.to_string()))
    }

    pub async fn find_window_bucket(&self) -> Result<Option<String>, AwError> {
        let buckets = self.get_buckets().await?;
        Ok(buckets
            .keys()
            .find(|k| k.starts_with("aw-watcher-window"))
            .cloned())
    }

    pub async fn find_afk_bucket(&self) -> Result<Option<String>, AwError> {
        let buckets = self.get_buckets().await?;
        Ok(buckets
            .keys()
            .find(|k| k.starts_with("aw-watcher-afk"))
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aw_event_accessors() {
        let mut data = HashMap::new();
        data.insert("app".to_string(), serde_json::Value::String("VS Code".to_string()));
        data.insert("title".to_string(), serde_json::Value::String("main.rs".to_string()));

        let event = AwEvent {
            id: Some(1),
            timestamp: "2026-04-01T10:00:00".to_string(),
            duration: 300.0,
            data,
        };

        assert_eq!(event.app(), Some("VS Code"));
        assert_eq!(event.title(), Some("main.rs"));
        assert_eq!(event.status(), None);
    }

    #[test]
    fn test_afk_event_status() {
        let mut data = HashMap::new();
        data.insert("status".to_string(), serde_json::Value::String("not-afk".to_string()));

        let event = AwEvent {
            id: Some(2),
            timestamp: "2026-04-01T10:00:00".to_string(),
            duration: 60.0,
            data,
        };

        assert_eq!(event.status(), Some("not-afk"));
    }

    #[test]
    fn test_client_construction() {
        let client = AwClient::new("http://localhost:5600");
        assert_eq!(client.base_url, "http://localhost:5600");

        let client2 = AwClient::new("http://localhost:5600/");
        assert_eq!(client2.base_url, "http://localhost:5600");
    }
}
