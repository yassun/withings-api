use std::fmt::Debug;

use serde::Serialize;

use crate::error::handle_response;

const MEASURE_PATH: &str = "/measure";

#[derive(Debug, Clone)]
pub struct ApiCli {
    access_token: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct GetMeasRequest {
    pub action: String,        // TODO: enum
    pub meastype: Option<u64>, // TODO: enum
    pub meastypes: Option<Vec<u64>>,
    pub category: Option<u64>, // TODO: enum
    pub startdate: Option<u64>,
    pub enddate: Option<u64>,
    pub offset: Option<u64>,
    pub lastupdate: Option<u64>,
}

impl ApiCli {
    pub fn new(access_token: String, base_url: String) -> ApiCli {
        ApiCli {
            access_token,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_meas(&self, req: &GetMeasRequest) -> anyhow::Result<serde_json::value::Value> {
        let res = self
            .client
            .post(format!("{}{}", &self.base_url, MEASURE_PATH))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .form(&req)
            .send()
            .await?;

        Ok(handle_response(req, res).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_meas() -> anyhow::Result<()> {
        let client = ApiCli::new("access_token".into(), mockito::server_url());

        let req = GetMeasRequest {
            action: "getmeas".into(),
            meastype: Some(1),
            meastypes: None,
            category: Some(1),
            startdate: Some(1),
            enddate: Some(12345),
            offset: Some(1),
            lastupdate: Some(12345),
        };

        let response_body = json!({
            "status": 0,
            "body": {
              "updatetime": "1644138861",
              "timezone": "Asia/Tokyo",
              "measuregrps": [
                {
                  "grpid": 123456789,
                  "attrib": 0,
                  "date": 1643969671,
                  "created": 1643969717,
                  "category": 1,
                  "deviceid": "cc50f32653df14137da15aaaaa7b2e07",
                  "hash_deviceid": "f32bbbb318f14137da157b2e07",
                  "measures": [
                    {
                      "value": 80000,
                      "type": 1,
                      "unit": -3,
                      "algo": 3,
                      "fm": 131,
                      "apppfmid": 7,
                      "appliver": 5080201
                    }
                  ],
                  "comment": "test comment"
                }
              ],
              "more": 1,
              "offset": 0,
            }
        });

        let mock = mockito::mock("POST", MEASURE_PATH)
            .with_status(200)
            .match_body(serde_urlencoded::to_string(&req)?.as_str())
            .with_body(serde_json::to_string(&response_body)?)
            .create();

        let res = client.get_meas(&req).await?;
        assert_eq!(res, response_body);
        mock.assert();

        Ok(())
    }
}
