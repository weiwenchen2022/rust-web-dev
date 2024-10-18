use clap::Parser;
use std::env;

use handle_errors::Error;

/// Q&A web service API
#[derive(Debug, Parser, PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn or error)
    #[arg(short, long, default_value = "warn")]
    pub log_level: String,

    /// Which PORT the server is listening to
    #[arg(short, long, default_value = "3030")]
    pub port: u16,

    /// Database user
    #[arg(long, default_value = "postgres")]
    pub db_user: String,

    /// Database user
    #[arg(long, default_value = "")]
    pub db_password: String,

    /// URL for the postgres database
    #[arg(long, default_value = "localhost")]
    pub db_host: String,

    /// PORT number for the database connection
    #[arg(long, default_value = "5433")]
    pub db_port: u16,

    /// Database name
    #[arg(long, default_value = "rustwebdev")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Self, Error> {
        let config = Config::parse();

        let _ =
            env::var("BAD_WORDS_API_KEY").unwrap_or_else(|_| panic!("BadWords API key not set"));
        let _ = env::var("PASETO_KEY").unwrap_or_else(|_| panic!("PASETO key not set"));

        let port = env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .transpose()
            .map_err(Error::ParseError)?
            .unwrap_or(config.port);

        let db_user = env::var("POSTGRES_USER").unwrap_or(config.db_user);
        let db_password = env::var("POSTGRES_PASSWORD").unwrap();
        let db_host = env::var("POSTGRES_HOST").unwrap_or(config.db_host);
        let db_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| config.db_port.to_string());
        let db_name = env::var("POSTGRES_DB").unwrap_or(config.db_name);

        Ok(Self {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port.parse::<u16>().map_err(Error::ParseError)?,
            db_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_and_set_api_key() {
        let result = std::panic::catch_unwind(Config::new);
        assert!(result.is_err());

        set_env();
        let config = Config::new().unwrap();
        let expected = Config {
            log_level: "warn".to_string(),
            port: 3030,
            db_user: "postgres".to_string(),
            db_password: "pass".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "rustwebdev".to_string(),
        };

        assert_eq!(expected, config);
    }

    fn set_env() {
        env::set_var("BAD_WORDS_API_KEY", "yes");
        env::set_var("POSTGRES_USER", "postgres");
        env::set_var("POSTGRES_PASSWORD", "pass");
        env::set_var("POSTGRES_HOST", "localhost");
        env::set_var("POSTGRES_PORT", "5432");
        env::set_var("POSTGRES_DB", "rustwebdev");
    }
}
