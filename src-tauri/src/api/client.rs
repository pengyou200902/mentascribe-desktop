use super::{AuthToken, UserInfo};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const API_BASE_URL: &str = "https://api.voice.mentaflux.ai/v1";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Unauthorized")]
    Unauthorized,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
    user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    message: String,
}

pub async fn login(email: &str, password: &str) -> Result<AuthToken, ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/auth/login", API_BASE_URL))
        .json(&LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        })
        .send()
        .await
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    if response.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !response.status().is_success() {
        let error: ApiErrorResponse = response
            .json()
            .await
            .unwrap_or(ApiErrorResponse {
                message: "Unknown error".to_string(),
            });
        return Err(ApiError::ApiError(error.message));
    }

    let login_response: LoginResponse = response
        .json()
        .await
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    Ok(AuthToken {
        access_token: login_response.access_token,
        refresh_token: login_response.refresh_token,
        expires_in: login_response.expires_in,
        user: login_response.user,
    })
}

pub async fn refresh_token(refresh_token: &str) -> Result<AuthToken, ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/auth/refresh", API_BASE_URL))
        .json(&serde_json::json!({ "refreshToken": refresh_token }))
        .send()
        .await
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    if response.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !response.status().is_success() {
        let error: ApiErrorResponse = response
            .json()
            .await
            .unwrap_or(ApiErrorResponse {
                message: "Unknown error".to_string(),
            });
        return Err(ApiError::ApiError(error.message));
    }

    let login_response: LoginResponse = response
        .json()
        .await
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    Ok(AuthToken {
        access_token: login_response.access_token,
        refresh_token: login_response.refresh_token,
        expires_in: login_response.expires_in,
        user: login_response.user,
    })
}

#[derive(Debug, Serialize)]
struct CreateTranscriptionRequest {
    #[serde(rename = "rawText")]
    raw_text: String,
    #[serde(rename = "cleanedText")]
    cleaned_text: Option<String>,
    #[serde(rename = "durationMs")]
    duration_ms: Option<u64>,
    language: Option<String>,
}

pub async fn create_transcription(
    access_token: &str,
    raw_text: &str,
    cleaned_text: Option<&str>,
    duration_ms: Option<u64>,
    language: Option<&str>,
) -> Result<(), ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/transcriptions", API_BASE_URL))
        .bearer_auth(access_token)
        .json(&CreateTranscriptionRequest {
            raw_text: raw_text.to_string(),
            cleaned_text: cleaned_text.map(|s| s.to_string()),
            duration_ms,
            language: language.map(|s| s.to_string()),
        })
        .send()
        .await
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    if response.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !response.status().is_success() {
        let error: ApiErrorResponse = response
            .json()
            .await
            .unwrap_or(ApiErrorResponse {
                message: "Unknown error".to_string(),
            });
        return Err(ApiError::ApiError(error.message));
    }

    Ok(())
}

/// Store tokens securely in OS keychain
pub fn store_tokens(access_token: &str, refresh_token: &str) -> Result<(), ApiError> {
    let entry = keyring::Entry::new("mentascribe", "tokens")
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    let tokens = serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
    });

    entry
        .set_password(&tokens.to_string())
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    Ok(())
}

/// Retrieve tokens from OS keychain
pub fn get_stored_tokens() -> Result<(String, String), ApiError> {
    let entry = keyring::Entry::new("mentascribe", "tokens")
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    let password = entry
        .get_password()
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    let tokens: serde_json::Value = serde_json::from_str(&password)
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    let access_token = tokens["access_token"]
        .as_str()
        .ok_or_else(|| ApiError::RequestError("Invalid token format".to_string()))?;

    let refresh_token = tokens["refresh_token"]
        .as_str()
        .ok_or_else(|| ApiError::RequestError("Invalid token format".to_string()))?;

    Ok((access_token.to_string(), refresh_token.to_string()))
}

/// Clear stored tokens
pub fn clear_tokens() -> Result<(), ApiError> {
    let entry = keyring::Entry::new("mentascribe", "tokens")
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    entry
        .delete_password()
        .map_err(|e| ApiError::RequestError(e.to_string()))?;

    Ok(())
}
