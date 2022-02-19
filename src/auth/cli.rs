use std::fmt::Debug;

use serde::de;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

use crate::error::handle_response;

const AUTH2_TOKEN_PATH: &str = "/v2/oauth2";

pub const WITHINGS_ACCOUNT_URL: &str = "https://account.withings.com";
const AUTHORIZE_PATH: &str = "/oauth2_user/authorize2";

#[derive(Debug, Clone)]
pub struct AuthCli {
    pub response_type: String,
    pub client_id: String,
    pub consumer_secret: String,
    pub callback_uri: String,
    pub scope: Vec<Scope>,
    pub mode: Option<String>,
    pub client: reqwest::Client,
    pub base_api_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccessTokenRequest {
    pub action: String,
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AccessTokenResponse {
    pub status: u64,
    pub body: AccessToken,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AccessToken {
    #[serde(rename = "userid")]
    pub user_id: u64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    #[serde(deserialize_with = "deserialize_vec_scope")]
    pub scope: Vec<Scope>,
    pub token_type: String,
}

fn deserialize_vec_scope<'de, D>(deserializer: D) -> Result<Vec<Scope>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .split(',')
        .map(Scope::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RefreshTokenRequest {
    pub action: String,
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RefreshTokenResponse {
    pub status: u64,
    pub body: RefreshToken,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RefreshToken {
    #[serde(rename = "userid")]
    pub user_id: u64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    #[serde(deserialize_with = "deserialize_vec_scope")]
    pub scope: Vec<Scope>,
    pub token_type: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Deserialize,
    strum_macros::EnumString,
    strum_macros::Display,
    strum_macros::IntoStaticStr,
    strum_macros::EnumIter,
)]
pub enum Scope {
    #[strum(serialize = "user.info")]
    UserInfo,
    #[strum(serialize = "user.metrics")]
    UserMetrics,
    #[strum(serialize = "user.activity")]
    UserActivity,
    #[strum(serialize = "user.sleepevents")]
    UserSleepEvents,
}

impl AuthCli {
    pub fn new(
        base_api_url: String,
        client_id: String,
        consumer_secret: String,
        callback_uri: String,
        scope: Vec<Scope>,
        mode: Option<String>,
    ) -> AuthCli {
        AuthCli {
            base_api_url,
            client_id,
            consumer_secret,
            callback_uri,
            scope,
            mode,
            response_type: "code".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn get_authorize_url(&self) -> anyhow::Result<String> {
        let mut q = vec![
            ("response_type", &self.response_type),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.callback_uri),
        ];

        let s = &self
            .scope
            .iter()
            .map(|x| x.into())
            .collect::<Vec<&str>>()
            .join(",");

        if !s.is_empty() {
            q.push(("scope", s));
        }

        let default_mode: String = "dev".into();
        match &self.mode {
            Some(m) => q.push(("state", m)),
            None => q.push(("state", &default_mode)),
        }

        let url =
            Url::parse_with_params(&format!("{}{}", WITHINGS_ACCOUNT_URL, AUTHORIZE_PATH), q)?;
        Ok(url.into())
    }

    pub async fn get_access_token(&self, code: &str) -> anyhow::Result<AccessTokenResponse> {
        let req = AccessTokenRequest {
            action: "requesttoken".into(),
            grant_type: "authorization_code".into(),
            client_id: self.client_id.clone(),
            client_secret: self.consumer_secret.clone(),
            redirect_uri: self.callback_uri.clone(),
            code: code.into(),
        };

        let res = self
            .client
            .post(format!("{}{}", &self.base_api_url, AUTH2_TOKEN_PATH))
            .form(&req)
            .send()
            .await?;

        Ok(handle_response(req, res).await?)
    }

    pub async fn get_refresh_token(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<RefreshTokenResponse> {
        let req = RefreshTokenRequest {
            action: "requesttoken".into(),
            grant_type: "refresh_token".into(),
            client_id: self.client_id.clone(),
            client_secret: self.consumer_secret.clone(),
            refresh_token: refresh_token.into(),
        };

        let res = self
            .client
            .post(format!("{}{}", &self.base_api_url, AUTH2_TOKEN_PATH))
            .form(&req)
            .send()
            .await?;

        Ok(handle_response(req, res).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::WithingsApiError, WITHINGS_API_URL};
    use assert_matches::assert_matches;
    use reqwest::StatusCode;
    use rstest::rstest;
    use serde_json::json;

    #[test]
    fn test_scope() -> anyhow::Result<()> {
        let scope = vec![Scope::UserInfo, Scope::UserMetrics];
        let s = scope
            .iter()
            .map(|x| x.into())
            .collect::<Vec<&str>>()
            .join(",");
        assert_eq!(s, "user.info,user.metrics");
        Ok(())
    }

    #[rstest]
    #[case(
        "test_client_id".into(),
        "test_consumer_secret".into(),
        "https://localhost".into(),
        vec![Scope::UserInfo, Scope::UserMetrics],
        Some("mode".into()),
        "https://account.withings.com/oauth2_user/authorize2?response_type=code&client_id=test_client_id&redirect_uri=https%3A%2F%2Flocalhost&scope=user.info%2Cuser.metrics&state=mode"
    )]
    #[case(
        "test_client_id".into(),
        "test_consumer_secret".into(),
        "https://localhost".into(),
        vec![],
        None,
        "https://account.withings.com/oauth2_user/authorize2?response_type=code&client_id=test_client_id&redirect_uri=https%3A%2F%2Flocalhost&state=dev"
    )]
    fn test_get_authorize_url(
        #[case] test_client_id: String,
        #[case] test_consumer_secret: String,
        #[case] test_callback_uri: String,
        #[case] scope: Vec<Scope>,
        #[case] mode: Option<String>,
        #[case] expected: &str,
    ) -> anyhow::Result<()> {
        let client = AuthCli::new(
            WITHINGS_API_URL.into(),
            test_client_id,
            test_consumer_secret,
            test_callback_uri,
            scope,
            mode,
        );

        assert_eq!(client.get_authorize_url().unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_desrialize_access_token_response() -> anyhow::Result<()> {
        let json = serde_json::to_string(&json!({
            "status": 0,
            "body": {
                "userid": 363,
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token",
                "expires_in": 10800,
                "scope": "user.info,user.metrics",
                "token_type": "Bearer"
            }
        }))?;

        let data: AccessTokenResponse = serde_json::from_str(&json)?;
        assert_eq!(
            data,
            AccessTokenResponse {
                status: 0,
                body: AccessToken {
                    user_id: 363,
                    access_token: "test_access_token".into(),
                    refresh_token: "test_refresh_token".into(),
                    expires_in: 10800,
                    scope: vec![Scope::UserInfo, Scope::UserMetrics],
                    token_type: "Bearer".into()
                }
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_token() -> anyhow::Result<()> {
        let client = AuthCli::new(
            mockito::server_url(),
            "test_client_id".into(),
            "test_consumer_secret".into(),
            "https://localhost".into(),
            vec![Scope::UserInfo, Scope::UserMetrics],
            Some("mode".into()),
        );

        let code = "sample_authorization_code";

        let req = serde_urlencoded::to_string(AccessTokenRequest {
            action: "requesttoken".into(),
            grant_type: "authorization_code".into(),
            client_id: client.client_id.clone(),
            client_secret: client.consumer_secret.clone(),
            redirect_uri: client.callback_uri.clone(),
            code: code.into(),
        })?;

        let mock = mockito::mock("POST", AUTH2_TOKEN_PATH)
            .with_status(200)
            .match_body(req.as_str())
            .with_body(serde_json::to_string(&json!({
                "status": 0,
                "body": {
                    "userid": 363,
                    "access_token": "test_access_token",
                    "refresh_token": "test_refresh_token",
                    "expires_in": 10800,
                    "scope": "user.info,user.metrics",
                    "token_type": "Bearer"
                }
            }))?)
            .create();

        let response_body = AccessTokenResponse {
            status: 0,
            body: AccessToken {
                user_id: 363,
                access_token: "test_access_token".into(),
                refresh_token: "test_refresh_token".into(),
                expires_in: 10800,
                scope: vec![Scope::UserInfo, Scope::UserMetrics],
                token_type: "Bearer".into(),
            },
        };

        let res = client.get_access_token(code).await?;
        assert_eq!(res, response_body);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_token_internal_server_error() -> anyhow::Result<()> {
        let client = AuthCli::new(
            mockito::server_url(),
            "test_client_id".into(),
            "test_consumer_secret".into(),
            "https://localhost".into(),
            vec![Scope::UserInfo, Scope::UserMetrics],
            Some("mode".into()),
        );

        let code = "sample_authorization_code";

        let req = serde_urlencoded::to_string(AccessTokenRequest {
            action: "requesttoken".into(),
            grant_type: "authorization_code".into(),
            client_id: client.client_id.clone(),
            client_secret: client.consumer_secret.clone(),
            redirect_uri: client.callback_uri.clone(),
            code: code.into(),
        })?;

        let mock = mockito::mock("POST", AUTH2_TOKEN_PATH)
            .with_status(500)
            .match_body(req.as_str())
            .create();

        let res = client.get_access_token(code).await;

        assert_matches!(res, Err(err) => {
            if let Some(WithingsApiError::InternalServerError{status, ..}) = err.downcast_ref::<WithingsApiError>() {
                assert_eq!(*status, StatusCode::INTERNAL_SERVER_ERROR);
            } else {
                panic!("is not WithingsError");
            }
        });

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_get_access_token_invalid_params_error() -> anyhow::Result<()> {
        let client = AuthCli::new(
            mockito::server_url(),
            "test_client_id".into(),
            "test_consumer_secret".into(),
            "https://localhost".into(),
            vec![Scope::UserInfo, Scope::UserMetrics],
            Some("mode".into()),
        );

        let code = "sample_authorization_code";

        let req = serde_urlencoded::to_string(AccessTokenRequest {
            action: "requesttoken".into(),
            grant_type: "authorization_code".into(),
            client_id: client.client_id.clone(),
            client_secret: client.consumer_secret.clone(),
            redirect_uri: client.callback_uri.clone(),
            code: code.into(),
        })?;

        let mock = mockito::mock("POST", AUTH2_TOKEN_PATH)
            .with_status(200)
            .match_body(req.as_str())
            .with_body(serde_json::to_string(&json!({
                "status": 503,
                "body": {}
            }))?)
            .create();

        let res = client.get_access_token(code).await;

        assert_matches!(res, Err(err) => {
            if let Some(WithingsApiError::InvalidParams{status, ..}) = err.downcast_ref::<WithingsApiError>() {
                assert_eq!(*status, 503);
            } else {
                panic!("is not WithingsApiError");
            }
        });

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn test_get_refresh_token() -> anyhow::Result<()> {
        let client = AuthCli::new(
            mockito::server_url(),
            "test_client_id".into(),
            "test_consumer_secret".into(),
            "https://localhost".into(),
            vec![Scope::UserInfo, Scope::UserMetrics],
            Some("mode".into()),
        );

        let refresh_token = "sample_refresh_token";

        let req = serde_urlencoded::to_string(RefreshTokenRequest {
            action: "requesttoken".into(),
            grant_type: "refresh_token".into(),
            client_id: client.client_id.clone(),
            client_secret: client.consumer_secret.clone(),
            refresh_token: refresh_token.into(),
        })?;

        let mock = mockito::mock("POST", AUTH2_TOKEN_PATH)
            .with_status(200)
            .match_body(req.as_str())
            .with_body(serde_json::to_string(&json!({
                "status": 0,
                "body": {
                    "userid": 363,
                    "access_token": "test_access_token",
                    "refresh_token": "test_refresh_token",
                    "expires_in": 10800,
                    "scope": "user.info,user.metrics",
                    "token_type": "Bearer"
                }
            }))?)
            .create();

        let response_body = RefreshTokenResponse {
            status: 0,
            body: RefreshToken {
                user_id: 363,
                access_token: "test_access_token".into(),
                refresh_token: "test_refresh_token".into(),
                expires_in: 10800,
                scope: vec![Scope::UserInfo, Scope::UserMetrics],
                token_type: "Bearer".into(),
            },
        };

        let res = client.get_refresh_token(refresh_token).await?;
        assert_eq!(res, response_body);
        mock.assert();

        Ok(())
    }
}
