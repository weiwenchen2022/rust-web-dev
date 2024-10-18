#![warn(clippy::all)]

mod routes;
mod store;
mod types;

mod profanity;

mod config;

use warp::http::Method;
use warp::reply::Reply;
use warp::Filter;

use store::Store;

use tracing_subscriber::fmt::format::FmtSpan;

pub use config::Config;

pub use handle_errors::Error;

use tokio::sync::oneshot::{self, Sender};

use std::net::SocketAddr;

pub struct OneshotHandler {
    pub sender: Sender<()>,
    pub bind_addr: SocketAddr,
}

pub async fn oneshot(store: Store) -> OneshotHandler {
    let routes = build_routes(store);
    let (tx, rx) = oneshot::channel();

    let bind_addr: SocketAddr = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Not a valid address")
        .local_addr()
        .expect("Not a valid address");
    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(bind_addr, async {
        let _ = rx.await;
    });

    tokio::task::spawn(server);

    OneshotHandler {
        sender: tx,
        bind_addr,
    }
}

pub async fn run(config: Config, store: Store) {
    let routes = build_routes(store);
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;
}

fn build_routes(store: Store) -> impl Filter<Extract = impl Reply> + Clone {
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::auth())
        .and(store_filter.clone())
        .and_then(routes::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::login);

    get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error)
}

pub async fn setup_store(config: &Config) -> Result<Store, Error> {
    let store = Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name,
    ))
    .await
    .map_err(Error::DatabaseQueryError)?;

    sqlx::migrate!()
        .run(&store.connection)
        .await
        .map_err(Error::MigrationError)?;

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "{}={},handle_errors={},warp={}",
            env!("CARGO_CRATE_NAME"),
            config.log_level,
            config.log_level,
            config.log_level,
        )
    });

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when each span closes.
        // This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}
