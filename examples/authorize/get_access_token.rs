//! `cargo run --example get_access_token --features=env`
#![deny(warnings)]

use dotenv::dotenv;
use withings_api::{
    auth::cli::{AuthCli, Scope},
    WITHINGS_API_URL,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = AuthCli::new(
        WITHINGS_API_URL.into(),
        std::env::var("CLIENT_ID").expect("CLIENT_ID must be present."),
        std::env::var("CONSUMER_SECRET").expect("CONSUMER_SECRET must be present."),
        std::env::var("CALLBACK_URL").expect("CALLBACK_URL must be present."),
        vec![Scope::UserInfo, Scope::UserMetrics],
        Some("demo".into()),
    );

    let res = client
        .get_access_token(&std::env::var("CODE").expect("CODE must be present."))
        .await
        .unwrap_or_else(|e| panic!("error {}", e));

    println!("Response {:?}", res);
}
