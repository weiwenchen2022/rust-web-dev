use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::{NewAnswer, Session};

use warp::http::StatusCode;
use warp::{Rejection, Reply};

use handle_errors::Error;

pub async fn add_answer(
    session: Session,
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if !store
        .is_question_owner(new_answer.question_id.0, &account_id)
        .await?
    {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    let content = check_profanity(new_answer.content)
        .await
        .map_err(warp::reject::custom)?;
    let answer = NewAnswer {
        content,
        ..new_answer
    };

    store
        .add_answer(answer, account_id)
        .await
        .map(|_| warp::reply::with_status("Answer added", StatusCode::OK))
        .map_err(warp::reject::custom)
}
