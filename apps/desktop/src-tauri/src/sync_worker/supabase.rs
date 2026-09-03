use reqwest::Client;
use serde_json::Value;

pub struct SupabaseClient {
    client: Client,
    api_url: String,
    anon_key: String,
    jwt: Option<String>,
}

impl SupabaseClient {
    pub fn new(api_url: String, anon_key: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
            anon_key,
            jwt: None,
        }
    }

    pub fn set_jwt(&mut self, jwt: String) {
        self.jwt = Some(jwt);
    }

    pub fn has_auth(&self) -> bool {
        self.jwt.is_some()
    }

    /// Upserts a batch of activity sessions. Uses `Prefer: resolution=merge-duplicates` 
    /// which relies on the `ON CONFLICT (id) DO UPDATE` behavior in PostgREST.
    pub async fn upsert_activity_sessions(&self, mut sessions: Vec<Value>) -> Result<(), String> {
        let url = format!("{}/rest/v1/activity_sessions", self.api_url);
        
        let jwt = if let Some(jwt) = &self.jwt {
            jwt
        } else {
            return Err("Cannot sync without authentication".into());
        };

        // Extract user_id from JWT
        let mut user_id = None;
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() == 3 {
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
                if let Ok(json) = serde_json::from_slice::<Value>(&decoded) {
                    if let Some(sub) = json.get("sub").and_then(|v| v.as_str()) {
                        user_id = Some(sub.to_string());
                    }
                }
            }
        }

        if let Some(uid) = user_id {
            for session in &mut sessions {
                if let Some(obj) = session.as_object_mut() {
                    obj.insert("user_id".to_string(), Value::String(uid.clone()));
                }
            }
        }

        let req = self.client.post(&url)
            .header("apikey", &self.anon_key)
            .header("Prefer", "resolution=merge-duplicates")
            .header("Authorization", format!("Bearer {}", jwt))
            .json(&sessions);

        let resp = req.send().await.map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Supabase error {}: {}", status, text))
        }
    }

    pub async fn get_settings(&self, updated_after: Option<&str>, last_sync_id: Option<&str>) -> Result<Vec<Value>, String> {
        let mut url = format!("{}/rest/v1/settings", self.api_url);
        if let Some(cursor) = updated_after {
            if let Some(id_cursor) = last_sync_id {
                url.push_str(&format!("?or=(updated_at.gt.{},and(updated_at.eq.{},id.gt.{}))", cursor, cursor, id_cursor));
            } else {
                url.push_str(&format!("?updated_at=gt.{}", cursor));
            }
        }

        let mut req = self.client.get(&url)
            .header("apikey", &self.anon_key);

        if let Some(jwt) = &self.jwt {
            req = req.header("Authorization", format!("Bearer {}", jwt));
        }

        let resp = req.send().await.map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if resp.status().is_success() {
            resp.json::<Vec<Value>>().await.map_err(|e| format!("JSON parse error: {}", e))
        } else {
            Err(format!("Supabase GET error: {}", resp.status()))
        }
    }

    pub async fn get_tombstones(&self, updated_after: Option<&str>, last_sync_id: Option<&str>) -> Result<Vec<Value>, String> {
        let mut url = format!("{}/rest/v1/tombstones", self.api_url);
        if let Some(cursor) = updated_after {
            if let Some(id_cursor) = last_sync_id {
                url.push_str(&format!("?or=(updated_at.gt.{},and(updated_at.eq.{},id.gt.{}))", cursor, cursor, id_cursor));
            } else {
                url.push_str(&format!("?updated_at=gt.{}", cursor));
            }
        }

        let mut req = self.client.get(&url)
            .header("apikey", &self.anon_key);

        if let Some(jwt) = &self.jwt {
            req = req.header("Authorization", format!("Bearer {}", jwt));
        }

        let resp = req.send().await.map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if resp.status().is_success() {
            resp.json::<Vec<Value>>().await.map_err(|e| format!("JSON parse error: {}", e))
        } else {
            Err(format!("Supabase GET error: {}", resp.status()))
        }
    }

    pub async fn refresh_token(&mut self, refresh_token: &str) -> Result<String, String> {
        let url = format!("{}/auth/v1/token?grant_type=refresh_token", self.api_url);
        
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "refresh_token": refresh_token
        });

        let res = client.post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = res.status();
        if status.is_success() {
            let body: serde_json::Value = res.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
            if let Some(access_token) = body.get("access_token").and_then(|t| t.as_str()) {
                self.set_jwt(access_token.to_string());
                return Ok(access_token.to_string());
            }
        }
        
        Err(format!("Failed to refresh token: HTTP {}", status))
    }
}
