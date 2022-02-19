//! `cargo run --example getmeas --features=env`
#![deny(warnings)]

use dotenv::dotenv;
use withings_api::{
    api::cli::{ApiCli, GetMeasRequest},
    WITHINGS_API_URL,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = ApiCli::new(
        std::env::var("ACCSESS_TOKEN").expect("ACCSESS_TOKEN  must be present."),
        WITHINGS_API_URL.into(),
    );
    let req = GetMeasRequest {
        action: "getmeas".into(),
        ..Default::default()
    };
    let res = client
        .get_meas(&req)
        .await
        .unwrap_or_else(|e| panic!("error {}", e));

    println!("Response {:?}", res);
}
