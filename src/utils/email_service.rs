use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::models::Invitation;
use crate::utils::errors::ServiceError;

pub fn send_mail(invitation: &Invitation) -> Result<(), ServiceError> {
    let sending_username: String = std::env::var("USERNAME").expect("USERNAME must be set.");
    let sending_password: String = std::env::var("PASSWORD").expect("USERNAME must be set.");
    let sending_host = std::env::var("HOST").expect("USERNAME must be set.");

    let email_body = format!(
        "Please click on the link below to complete registration. <br/>
         <a href=\"http://localhost:9100/register.html?id={}&email={}\">
         http://localhost:9100/register</a> <br>
         your Invitation expires on <strong>{}</strong>",
        invitation.id,
        invitation.email,
        invitation
            .expires_at
            .format("%I:%M %p %A, %-d %B, %C%y")
            .to_string()
    );

    let mail_builder = Message::builder()
        .from(sending_username.parse().unwrap())
        .reply_to(sending_username.parse().unwrap())
        .to(invitation.email.parse().unwrap())
        .subject("Your invitation")
        .body(email_body)
        .unwrap();

    let creds = Credentials::new(sending_username, sending_password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&sending_host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&mail_builder) {
        Ok(_) => {
            println!("Email sent successfully!");
            Ok(())
        }
        Err(e) => {
            println!("Could not send email: {:?}", e);
            Err(ServiceError::InternalServerError)
        }
    }
}