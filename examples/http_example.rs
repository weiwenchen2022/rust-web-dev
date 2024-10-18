use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use serde_json::json;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character=*")
        .header("apikey", "wLgtZIGPFqkQoRy3U5OGERBILFKsAgwM")
        .body("a list with shit words")
        .send()
        .await?;

    // let res = res.error_for_status()?;
    let status_code = res.status();
    let message = res.text().await?;
    let response = json!({
        "StatusCode": status_code.as_str(),
        "Message": message,
    });
    println!("{:#?}", response);

    Ok(())
}
