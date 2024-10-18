use std::collections::HashMap;

use crate::store::Store;
use crate::types::{extract_pagination, NewQuestion, Pagination, Question, Session};

use handle_errors::Error;
use warp::http::StatusCode;

use crate::profanity::check_profanity;
use warp::{Rejection, Reply};

use tracing::{event, info, instrument, Level};

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    event!(target: env!("CARGO_CRATE_NAME"), Level::INFO, "querying questions");

    let pagination = if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        extract_pagination(&params)?
    } else {
        info!(pagination = false);
        Pagination::default()
    };

    let questions = store
        .get_questions(pagination.limit, pagination.offset)
        .await
        .map_err(warp::reject::custom)?;

    Ok(warp::reply::json(&questions))
}

pub async fn add_question(
    session: Session,
    store: Store,
    new_question: NewQuestion,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    let title = check_profanity(new_question.title);
    let content = check_profanity(new_question.content);
    let (title, content) = tokio::try_join!(title, content).map_err(warp::reject::custom)?;
    let new_question = NewQuestion {
        title,
        content,
        ..new_question
    };

    store
        .add_question(new_question, account_id)
        .await
        .map(|qustion| warp::reply::json(&qustion))
        .map_err(warp::reject::custom)
}

pub async fn update_question(
    id: i32,
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if !store.is_question_owner(id, &account_id).await? {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    let title = check_profanity(question.title);
    let content = check_profanity(question.content);
    let (title, content) = tokio::try_join!(title, content).map_err(warp::reject::custom)?;
    let question = Question {
        title,
        content,
        ..question
    };

    store
        .update_question(question, id, account_id)
        .await
        .map(|question| warp::reply::json(&question))
        .map_err(warp::reject::custom)
}

pub async fn delete_question(
    question_id: i32,
    session: Session,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if !store.is_question_owner(question_id, &account_id).await? {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    store
        .delete_question(question_id, account_id)
        .await
        .map(|_| {
            warp::reply::with_status(format!("Question {} deleted", question_id), StatusCode::OK)
        })
        .map_err(warp::reject::custom)
}
