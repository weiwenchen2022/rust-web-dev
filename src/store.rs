use handle_errors::Error;

use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;

use crate::types::{
    Account, AccountId, Answer, AnswerId, NewAnswer, NewQuestion, Question, QuestionId,
};

use tracing::{event, Level};

#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        tracing::warn!("{}", db_url);

        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        Ok(Self {
            connection: db_pool,
        })
    }
}

impl Store {
    pub async fn get_questions(
        &self,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Question>, Error> {
        sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.connection)
            .await
            .map_err(|err| {
                event!(Level::ERROR, "{:?}", err);
                Error::DatabaseQueryError(err)
            })
    }

    pub async fn add_question(
        &self,
        new_question: NewQuestion,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        let NewQuestion {
            title,
            content,
            tags,
        } = new_question;

        sqlx::query(
            "INSERT INTO questions (title, content, tags, account_id)
                VALUES ($1, $2, $3, $4)
                RETURNING id, title, content, tags",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await
        .map_err(|err| {
            event!(Level::ERROR, "{:?}", err);
            Error::DatabaseQueryError(err)
        })
    }

    pub async fn update_question(
        &self,
        question: Question,
        question_id: i32,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        println!("{}", account_id.0);

        let Question {
            id: _,
            title,
            content,
            tags,
        } = question;

        sqlx::query(
            "UPDATE questions
        SET title = $1, content = $2, tags = $3
        WHERE id = $4 AND account_id = $5
        RETURNING id, title, content, tags
        ",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .bind(question_id)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await
        .map_err(|err| {
            event!(Level::ERROR, "{:?}", err);
            Error::DatabaseQueryError(err)
        })
    }

    pub async fn delete_question(
        &self,
        question_id: i32,
        account_id: AccountId,
    ) -> Result<bool, Error> {
        sqlx::query("DELETE FROM questions WHERE id = $1 and account_id = $2")
            .bind(question_id)
            .bind(account_id.0)
            .execute(&self.connection)
            .await
            .map(|res| res.rows_affected() > 0)
            .map_err(|err| {
                event!(Level::ERROR, "{:?}", err);
                Error::DatabaseQueryError(err)
            })
    }

    pub async fn add_answer(
        &self,
        new_answer: NewAnswer,
        account_id: AccountId,
    ) -> Result<Answer, Error> {
        let NewAnswer {
            content,
            question_id,
        } = new_answer;

        sqlx::query("INSERT INTO answers (content, question_id, account_id) VALUES ($1, $2, $3)")
            .bind(content)
            .bind(question_id.0)
            .bind(account_id.0)
            .map(|row: PgRow| Answer {
                id: AnswerId(row.get("id")),
                content: row.get("content"),
                question_id: QuestionId(row.get("question_id")),
            })
            .fetch_one(&self.connection)
            .await
            .map_err(|err| {
                event!(Level::ERROR, "{:?}", err);
                Error::DatabaseQueryError(err)
            })
    }
}

impl Store {
    pub async fn add_account(&self, account: Account) -> Result<bool, Error> {
        let Account {
            email, password, ..
        } = account;

        sqlx::query(
            "INSERT INTO accounts (email, password)
        VALUES ($1, $2)",
        )
        .bind(email)
        .bind(password)
        .execute(&self.connection)
        .await
        .map(|_| true)
        .map_err(|err| {
            event!(
                Level::ERROR,
                code = err
                    .as_database_error()
                    .unwrap()
                    .code()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap(),
                db_message = err.as_database_error().unwrap().message(),
                constraint = err.as_database_error().unwrap().constraint().unwrap()
            );

            Error::DatabaseQueryError(err)
        })
    }

    pub async fn get_account(&self, email: String) -> Result<Account, Error> {
        sqlx::query("SELECT id,email,password from accounts WHERE email = $1")
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password"),
            })
            .fetch_one(&self.connection)
            .await
            .map_err(|err| {
                event!(Level::ERROR, "{:?}", err);
                Error::DatabaseQueryError(err)
            })
    }
}

impl Store {
    pub async fn is_question_owner(
        &self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        sqlx::query("SELECT 1 FROM questions WHERE id = $1 and account_id = $2")
            .bind(question_id)
            .bind(account_id.0)
            .fetch_optional(&self.connection)
            .await
            .map(|question| question.is_some())
            .map_err(|e| {
                event!(Level::ERROR, "{:?}", e);
                Error::DatabaseQueryError(e)
            })
    }
}
