use std::env;
use std::future;

use warp::{Filter, Rejection, Reply};

use crate::store::Store;
use crate::types::{Account, AccountId, Session};

use argon2::{self, Config};
use chrono::prelude::*;
use rand::Rng;

use handle_errors::Error;

pub async fn register(store: Store, account: Account) -> Result<impl Reply, Rejection> {
    let hashed_password = hashed_password(account.password.as_bytes());
    let account = Account {
        password: hashed_password,
        ..account
    };

    store
        .add_account(account)
        .await
        .map(|_| warp::reply::json(&"Account added".to_string()))
        .map_err(warp::reject::custom)
}

pub fn hashed_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

pub async fn login(store: Store, login: Account) -> Result<impl Reply, Rejection> {
    let account = store
        .get_account(login.email)
        .await
        .map_err(warp::reject::custom)?;

    match verify_password(&account.password, login.password.as_bytes()) {
        Ok(verified) => {
            if verified {
                Ok(warp::reply::json(&issue_token(
                    account.id.expect("id not found"),
                )))
            } else {
                Err(warp::reject::custom(Error::WrongPassword))
            }
        }

        Err(e) => Err(warp::reject::custom(Error::ArgonLibraryError(e))),
    }
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

fn issue_token(account_id: AccountId) -> String {
    let key = env::var("PASETO_KEY").unwrap();
    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(key.as_bytes())
        .set_expiration(&dt)
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

pub fn verify_token(token: String) -> Result<Session, Error> {
    let key = env::var("PASETO_KEY").unwrap();
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token).map_err(|_| Error::CannotDecryptToken)
}

pub fn auth() -> impl Filter<Extract = (Session,), Error = Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| match verify_token(token) {
        Ok(t) => future::ready(Ok(t)),
        Err(_) => future::ready(Err(warp::reject::reject())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn post_questions_auth() {
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        let token = issue_token(AccountId(3));

        let filter = auth();
        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);

        assert_eq!(AccountId(3), res.await.unwrap().account_id);
    }
}
