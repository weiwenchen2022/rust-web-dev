use crate::types::QuestionId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub i32);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewAnswer {
    pub content: String,
    pub question_id: QuestionId,
}
