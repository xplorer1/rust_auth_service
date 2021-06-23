use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use serde::Deserialize;

use crate::utils::{errors::ServiceError, responses::UserStruct, password_hash::hash_password, requests::UserData};
use crate::models::{Invitation, Pool, User};

pub async fn register_user(invitation_id: web::Path<String>, user_data: web::Json<UserData>, pool: web::Data<Pool>) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || run_query(invitation_id.into_inner(), user_data.into_inner(), pool)).await;

    match res {
        Ok(user) => Ok(HttpResponse::Ok().json(&user)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        }
    }
}

fn run_query(invitation_id: String, user_data: UserData, pool: web::Data<Pool>) -> Result<UserStruct, ServiceError> {
    use crate::schema::invitations::dsl::{email, id, invitations};
    use crate::schema::users::dsl::users;

    let invitation_id = uuid::Uuid::parse_str(&invitation_id)?;

    let conn: &PgConnection = &pool.get().unwrap();

    invitations
        .filter(id.eq(invitation_id))
        .filter(email.eq(&user_data.email))
        .load::<Invitation>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid invitation".into()))
        .and_then(|mut result| {
            if let Some(invitation) = result.pop() {
                //if invitation has not expired... 
                if invitation.expires_at > chrono::Local::now().naive_local() {
                    //try hashing the password
                    let password: String = hash_password(&user_data.password)?;
                    let user = User::from_details(invitation.email, password);

                    let inserted_user: User = diesel::insert_into(users).values(&user).get_result(conn)?;

                    dbg!(&inserted_user);

                    return Ok(inserted_user.into());
                }
            }

            Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}