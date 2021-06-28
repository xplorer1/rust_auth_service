use actix_identity::Identity;
use actix_web::{
    dev::Payload, error::BlockingError, web, Error, FromRequest,HttpRequest, HttpResponse
};

use diesel::prelude::*;
use diesel::PgConnection;
use futures::future::{err, ok, Ready};
use serde::Deserialize;

use crate::utils::{errors::ServiceError, responses::UserStruct};
use crate::models::{Pool, User};
use crate::utils::password_hash::verify_password;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String
}

pub type LoggedInUser = UserStruct;

impl FromRequest for LoggedInUser {
    type Config = ();
    type Error = Error;
    type Future = Ready<Result<LoggedInUser, Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, payload).into_inner() {
            if let Some(user_json) = identity.identity() {
                if let Ok(user) = serde_json::from_str(&user_json) {
                    return ok(user);
                }
            }
        }

        err(ServiceError::Unauthorized.into())
    }
}

pub async fn sign_out(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().finish()
}

pub async fn sign_in(auth_data: web::Json<AuthData>, id: Identity, pool: web::Data<Pool>) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || run_query(auth_data.into_inner(), pool)).await;

    match res {
        Ok(user) => {
            let user_string = serde_json::to_string(&user).unwrap();

            id.remember(user_string);
            Ok(HttpResponse::Ok().finish())
        }

        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        }
    }
}

pub async fn get_me(logged_in_user: LoggedInUser) -> HttpResponse {
    HttpResponse::Ok().json(logged_in_user)
}

fn run_query(auth_data: AuthData, pool: web::Data<Pool>) -> Result<UserStruct, ServiceError> {
    use crate::schema::users::dsl::{email, users};

    let conn: &PgConnection = &pool.get().unwrap();
    let mut items = users 
        .filter(email.eq(&auth_data.email))
        .load::<User>(conn)?;

    if let Some(user) = items.pop() {
        if let Ok(matching) = verify_password(&user.hash, &auth_data.password) {
            if matching {
                return Ok(user.into());
            }
        }
    }

    Err(ServiceError::Unauthorized)
}