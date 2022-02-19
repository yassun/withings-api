use std::fmt::Debug;

use reqwest::Response;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

#[derive(thiserror::Error, Debug)]
pub enum WithingsApiError {
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    AuthenticationFailed {
        status: u64,
        request: String,
        response: String,
    },
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    InvalidParams {
        status: u64,
        request: String,
        response: String,
    },
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    NotImplemented {
        status: u64,
        request: String,
        response: String,
    },
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    UnAuthorized {
        status: u64,
        request: String,
        response: String,
    },
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    AnErrorOccurred {
        status: u64,
        request: String,
        response: String,
    },
    #[error("Status: {status} Req: {request:#?} Res: {response:#?}")]
    InternalServerError {
        status: StatusCode,
        request: String,
        response: String,
    },
}

pub async fn handle_response<T: Debug, U: DeserializeOwned>(
    req: T,
    res: Response,
) -> anyhow::Result<U> {
    if !res.status().is_success() {
        return Err(WithingsApiError::InternalServerError {
            status: res.status(),
            request: format!("{:#?}", req),
            response: format!("{:#?}", res.text().await?),
        }
        .into());
    }

    let value: serde_json::Value = res.json().await?;
    return if let Some(body_status) = value["status"].as_u64() {
        // https://developer.withings.com/api-reference#section/Response-status
        match body_status {
            0 => Ok(serde_json::from_value(value)?),
            100..=102 | 200 => Err(WithingsApiError::AuthenticationFailed {
                status: body_status,
                request: format!("{:#?}", req),
                response: format!("{:#?}", value),
            }
            .into()),
            501..=511 => Err(WithingsApiError::InvalidParams {
                status: body_status,
                request: format!("{:#?}", req),
                response: format!("{:#?}", value),
            }
            .into()),
            _ => Err(anyhow::anyhow!(
                "Something else happened. Status: {:?} Req: {:#?} Res: {:#?}",
                body_status,
                req,
                value
            )),
        }
    } else {
        Err(anyhow::anyhow!(
            "Body status couldn't be parsed. Req: {:#?} Res: {:#?}",
            req,
            value
        ))
    };
}
