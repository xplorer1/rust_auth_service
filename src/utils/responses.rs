use crate::models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStruct {
    pub email: String
}

impl From<User> for UserStruct {
    fn from(user: User) -> Self {
        UserStruct { email: user.email }
    }
}