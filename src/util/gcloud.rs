use std::time::{Duration, Instant};

use crate::{config::ENV, error::AppError, models::gcloud::GCloudSrvAccountKey};
use base64::{Engine as _, engine::general_purpose};
use envmode::EnvMode;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Deserialize, Serialize)]
struct GenerateIdTokenRequest {
    audience: String,
    #[serde(rename = "includeEmail")]
    include_email: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateIdTokenResponse {
    token: String,
}

#[derive(Debug)]
struct CachedIdToken {
    token: String,
    expires_at: Instant,
}

static GCLOUD_IDTOKEN_CACHE: Lazy<RwLock<Option<CachedIdToken>>> = Lazy::new(|| RwLock::new(None));
static GCLOUD_IDTOKEN_REFRESH_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub async fn idtoken() -> Result<String, AppError> {
    {
        let cache = GCLOUD_IDTOKEN_CACHE.read().await;
        if let Some(cached) = cache.as_ref()
            && Instant::now() < cached.expires_at - Duration::from_secs(60)
        {
            return Ok(cached.token.clone());
        }
    }

    let _guard = GCLOUD_IDTOKEN_REFRESH_LOCK.lock().await;
    {
        let cache = GCLOUD_IDTOKEN_CACHE.read().await;
        if let Some(cached) = cache.as_ref()
            && Instant::now() < cached.expires_at - Duration::from_secs(60)
        {
            return Ok(cached.token.clone());
        }
    }

    let key = general_purpose::STANDARD.decode(&*ENV.gcloud_srv)?;
    let key = String::from_utf8(key)?;
    let key: GCloudSrvAccountKey =
        serde_json::from_str(&key).map_err(AppError::from_generic_error)?;

    let secret = yup_oauth2::ServiceAccountKey {
        key_type: Some(key.key_type),
        project_id: Some(key.project_id),
        private_key_id: Some(key.private_key_id),
        private_key: key.private_key,
        client_email: key.client_email,
        client_id: Some(key.client_id),
        auth_uri: Some(key.auth_uri),
        token_uri: key.token_uri,
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
    };

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("Failed to build authenticator: {:?}", e)))?;

    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
    let result = auth.token(scopes).await?;
    let token = result
        .token()
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("failed to get access token")))?;

    let domain = if EnvMode::is_prd(&ENV.environment) {
        &*ENV.grpc_prd_domain
    } else {
        &*ENV.grpc_dev_domain
    };
    let audience = format!("https://{}", domain);
    let url = format!(
        "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateIdToken",
        &*ENV.gcloud_srv_email
    );

    let body = GenerateIdTokenRequest {
        audience,
        include_email: true,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::Other(anyhow::anyhow!(
            "failed to generate id token[{}]: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        )));
    }
    let response: GenerateIdTokenResponse = response.json().await?;

    {
        let mut cache = GCLOUD_IDTOKEN_CACHE.write().await;
        *cache = Some(CachedIdToken {
            token: response.token.clone(),
            expires_at: Instant::now() + Duration::from_secs(59 * 60), // 59 minutes
        })
    }

    Ok(response.token)
}
