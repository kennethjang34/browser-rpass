use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Account {
    pub id: usize,
    pub username: Option<String>,
    pub email: String,
    pub password: Option<String>,
    pub organization: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
