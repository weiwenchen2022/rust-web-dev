use std::net::SocketAddr;

use rust_web_dev::{oneshot, setup_store, Config, Error};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use reqwest::Client;

use std::io::{self, Write};
use std::process::Command;

use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Token(String);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: Option<i32>,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct QuestionAnswer {
//     id: i32,
//     title: String,
//     content: String,
//     tags: Option<Vec<String>>,
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    let config = Config::new().expect("Config can't be set");

    let s = Command::new("sqlx")
        .arg("database")
        .arg("drop")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
        ))
        .arg("-y")
        .output()
        .expect("sqlx command failed to start");
    io::stderr().write_all(&s.stderr).unwrap();

    let s = Command::new("sqlx")
        .arg("database")
        .arg("create")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
        ))
        .output()
        .expect("sqlx command failed to start");
    io::stderr().write_all(&s.stderr).unwrap();

    let store = setup_store(&config).await?;
    let handler = oneshot(store).await;

    let u = User {
        email: "test@email.com".to_string(),
        password: "password".to_string(),
    };

    print!("Running register_new_user...");
    match AssertUnwindSafe(register_new_user(&u, handler.bind_addr))
        .catch_unwind()
        .await
    {
        Ok(_) => println!("✓"),
        Err(_) => {
            let _ = handler.sender.send(());
            std::process::exit(1);
        }
    }

    let token;
    print!("Running login...");
    match AssertUnwindSafe(login(&u, handler.bind_addr))
        .catch_unwind()
        .await
    {
        Ok(t) => {
            token = t;
            println!("✓");
        }
        Err(_) => {
            let _ = handler.sender.send(());
            std::process::exit(1);
        }
    }

    print!("Running post_question...");
    match AssertUnwindSafe(post_question(token, handler.bind_addr))
        .catch_unwind()
        .await
    {
        Ok(_) => println!("✓"),
        Err(_) => {
            let _ = handler.sender.send(());
            std::process::exit(1);
        }
    }

    let _ = handler.sender.send(());

    Ok(())
}

async fn register_new_user(user: &User, bind_addr: SocketAddr) {
    let client = Client::new();
    let res = client
        .post(format!("http://{}/registration", bind_addr))
        .json(user)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!("Account added", res);
}

async fn login(user: &User, bind_addr: SocketAddr) -> Token {
    let client = Client::new();
    let res = client
        .post(format!("http://{}/login", bind_addr))
        .json(user)
        .send()
        .await
        .unwrap();
    assert_eq!(200, res.status());
    res.json::<Token>().await.unwrap()
}

async fn post_question(token: Token, bind_addr: SocketAddr) {
    let q = Question {
        id: None,
        title: "First Question".to_string(),
        content: "How can I test?".to_string(),
        tags: None,
    };
    let client = Client::new();
    let res = client
        .post(format!("http://{}/questions", bind_addr))
        .header("Authorization", token.0)
        .json(&q)
        .send()
        .await
        .unwrap()
        .json::<Question>()
        .await
        .unwrap();

    assert_eq!(Some(1), res.id);
    assert_eq!(q.title, res.title);
    assert_eq!(q.content, res.content);
}
