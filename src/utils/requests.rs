use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub email: String,
    pub password: String,
}