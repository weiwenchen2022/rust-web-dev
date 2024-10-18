#![warn(clippy::all)]

use std::fmt::{self, Display};

use warp::filters::body::BodyDeserializeError;
use warp::filters::cors::CorsForbidden;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Rejection, Reply};

use tracing::{event, instrument, Level};

use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;

use argon2::Error as ArgonError;

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    DatabaseQueryError(sqlx::Error),

    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),

    ClientError(APILayerError),
    ServerError(APILayerError),

    WrongPassword,
    ArgonLibraryError(ArgonError),

    CannotDecryptToken,
    Unauthorized,

    MigrationError(sqlx::migrate::MigrateError),
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl Display for APILayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}

impl Reject for APILayerError {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),

            Error::DatabaseQueryError(_) => write!(f, "Cannot update, invalid data."),

            Error::ReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Error::MiddlewareReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Error::ClientError(err) => write!(f, "External Client error: {}", err),
            Error::ServerError(err) => write!(f, "External Server error: {}", err),

            Error::WrongPassword => write!(f, "Wrong password"),
            Error::ArgonLibraryError(_) => write!(f, "Cannot verifiy password"),

            Error::CannotDecryptToken => write!(f, "Cannot decrypt error"),
            Error::Unauthorized => write!(f, "No permission to change the underlying resource"),

            Error::MigrationError(_) => write!(f, "Cannot migrate data"),
        }
    }
}

impl Reject for Error {}

const DUPLICATE_KEY: u32 = 23505;

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "Database query error");

        match e {
            sqlx::Error::Database(err) => {
                if DUPLICATE_KEY == err.code().unwrap().parse::<u32>().unwrap() {
                    Ok(warp::reply::with_status(
                        "Account already exsists".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "Cannot update data".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
    } else if let Some(crate::Error::ReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::MiddlewareReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ClientError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ServerError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "Entered wrong password");
        Ok(warp::reply::with_status(
            "Wrong E-Mail/Password combination".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::Unauthorized) = r.find() {
        event!(Level::ERROR, "Not matching account id");
        Ok(warp::reply::with_status(
            "No permission to change underlying resource".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);

        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);

        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);

        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        event!(Level::WARN, "Requested route was not found");

        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
