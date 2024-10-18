use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use serde::{Deserialize, Serialize};

use handle_errors::{APILayerError, Error};

use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,

    #[serde(rename = "replacedLen")]
    replaced_len: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct APIResponse {
    message: String,
}

pub async fn check_profanity(content: String) -> Result<String, Error> {
    let api_key = env::var("BAD_WORDS_API_KEY").expect("BAD WORDS API KEY NOT SET");
    let api_layer_url = env::var("API_LAYER_URL").expect("APILAYER URL NOT SET");

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    let res = client
        .post(format!("{}/bad_words?censor_character=*", api_layer_url))
        .header("apikey", api_key)
        .body(content)
        .send()
        .await
        .map_err(Error::MiddlewareReqwestAPIError)?;

    if !res.status().is_success() {
        if res.status().is_client_error() {
            let err = transform_error(res).await;
            return Err(Error::ClientError(err));
        } else {
            let err = transform_error(res).await;
            return Err(Error::ServerError(err));
        }
    }

    res.json::<BadWordsResponse>()
        .await
        .map(|res| res.censored_content)
        .map_err(Error::ReqwestAPIError)
}

async fn transform_error(res: reqwest::Response) -> APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res.json::<APIResponse>().await.unwrap().message,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use mock_server::{MockServer, OneshotHandler};

    #[tokio::test]
    async fn run() {
        let handler = run_mock().await;
        censor_profane_words().await;
        no_profane_words().await;
        let _ = handler.sender.send(());
    }

    async fn run_mock() -> OneshotHandler {
        let mock = MockServer::new().await;
        env::set_var("API_LAYER_URL", format!("http://{}", mock.bind_addr));
        env::set_var("BAD_WORDS_API_KEY", "YES");

        mock.oneshot()
    }

    async fn censor_profane_words() {
        let content = "This is a shitty sentence".to_string();
        let censored_content = check_profanity(content).await.unwrap();
        assert_eq!("this is a ****** sentence", censored_content);
    }

    async fn no_profane_words() {
        let content = "this is a sentence".to_string();
        let censored_content = check_profanity(content).await.unwrap();
        assert_eq!("", censored_content);
    }
}
