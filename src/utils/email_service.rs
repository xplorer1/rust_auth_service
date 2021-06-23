use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::models::Invitation;
use crate::utils::errors::ServiceError;

pub fn send_mail(invitation: &Invitation) -> Result<(), ServiceError> {
    let sending_username: String = std::env::var("USERNAME").expect("USERNAME must be set.");
    let sending_password: String = std::env::var("PASSWORD").expect("USERNAME must be set.");
    let sending_host = std::env::var("HOST").expect("USERNAME must be set.");

    let mail_builder = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body(String::from("Be happy!"))
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