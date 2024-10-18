use serde_json::json;

use std::net::SocketAddr;

use tokio::sync::oneshot::{self, Sender};

use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use bytes::Bytes;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MockServer {
    pub bind_addr: SocketAddr,
}

pub struct OneshotHandler {
    pub sender: Sender<()>,
}

impl MockServer {
    pub async fn new() -> Self {
        let bind_addr = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Not a valid address")
            .local_addr()
            .expect("Not a valid address");
        Self { bind_addr }
    }

    async fn check_profanity(_: (), content: Bytes) -> Result<impl Reply, Rejection> {
        let content = String::from_utf8(content.to_vec()).expect("Invalid UTF-8");

        if content.contains("shitty") {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "bad_words_list": [
                    {
                        "deviations": 0,
                        "end": 16,
                        "info": 2,
                        "original": "shitty",
                        "replacedLen": 6,
                        "start": 10,
                        "word": "shitty"
                    }
                    ],
                    "bad_words_total": 1,
                    "censored_content": "this is a ****** sentence",
                    "content": "this is a shitty sentence"
                })),
                StatusCode::OK,
            ))
        } else {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "bad_words_list": [],
                    "bad_words_total": 0,
                    "censored_content": "",
                    "content": "this is a sentence"
                })),
                StatusCode::OK,
            ))
        }
    }

    fn build_routes(&self) -> impl Filter<Extract = impl Reply> + Clone {
        warp::post()
            .and(warp::path("bad_words"))
            .and(warp::query())
            .map(|_: HashMap<String, String>| ())
            .and(warp::path::end())
            .and(warp::body::bytes())
            .and_then(Self::check_profanity)
    }

    pub fn oneshot(&self) -> OneshotHandler {
        let (tx, rx) = oneshot::channel();
        let routes = Self::build_routes(self);

        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(self.bind_addr, async {
            rx.await.ok();
        });

        tokio::task::spawn(server);

        OneshotHandler { sender: tx }
    }
}
