use std::iter::Iterator;
use std::option::Option;
use std::option::Option::{None, Some};
use std::result::Result::{Err, Ok};
use std::string::{String, ToString};
use std::sync::{Arc, Mutex};
use std::thread;

use lettre::{
    Message,
    SmtpTransport, Transport, transport::smtp::authentication::Credentials,
};
use lettre::transport::smtp::extension::ClientId;

use mailparse::{MailHeaderMap, parse_mail, ParsedMail};

fn wait_for_email(sender: &str, smtp_password: &str, recipients: Vec<&str>) {
    let sender = sender.to_string();
    let smtp_password = smtp_password.to_string();
    let recipients = recipients.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let imap_server = "imap.gmail.com";
    let client = imap::connect((imap_server, 993), imap_server, &tls).unwrap();
    let mut imap_session = client
        .login(&sender, &smtp_password).unwrap();
    imap_session.select("INBOX").unwrap();
    let session = Arc::new(Mutex::new(imap_session));
    let last_email_uid: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    loop {
        let session_clone = Arc::clone(&session);
        let last_email_uid_clone = Arc::clone(&last_email_uid);
        let sender_clone = sender.clone();
        let smtp_password_clone = smtp_password.clone();
        let recipients_clone = recipients.clone();
        // Use a separate thread to run the idle command
        thread::spawn(move || -> imap::error::Result<()> {
            let mut session_lock = session_clone.lock().unwrap();
            let mut last_email_uid_lock = last_email_uid_clone.lock().unwrap();
            match session_lock.idle().expect("REASON").wait_keepalive() {
                Ok(_) => {
                    if let Ok(email) = fetch_last_email(&mut session_lock) {
                        if !email.is_empty() {
                            let uids = session_lock.search("ALL")?;
                            let new_email_uid = uids.iter().max();
                            if  last_email_uid_lock.is_none() || last_email_uid_lock.unwrap() != new_email_uid.copied().unwrap() {
                                for recipient in &recipients_clone {
                                    send_email(email.as_bytes(), &sender_clone, &smtp_password_clone, recipient);
                                }
                                *last_email_uid_lock = Some(new_email_uid).expect("REASON").copied();
                            }
                        }
                    }
                },
                Err(e) => eprintln!("Error in IDLE: {}", e),
            }
            Ok(())
        }).join().unwrap();
    }
}
fn fetch_last_email(session: &mut imap::Session<native_tls::TlsStream<std::net::TcpStream>>) -> imap::error::Result<String> {
    // Search for the last email
    let uids = session.search("ALL")?;
    let last_email_uid = uids.iter().max().unwrap();
    let messages = session.fetch(last_email_uid.to_string().as_str(), "RFC822")?;
    if let Some(message) = messages.iter().next() {
        if let Some(body) = message.body() {
            let email = std::str::from_utf8(body).unwrap_or("").to_string();
            return Ok(email);
        }
    }
    Err(imap::error::Error::Bad("No email found".to_string()))
}

fn send_email(email: &[u8], sender: &str, smtp_password: &str, recipient: &str) {
    // Parse the header
    let parsed_mail :ParsedMail = parse_mail(email).unwrap();
    let parsed_mail_headers = parsed_mail.get_headers();
    let parsed_mail_body = if parsed_mail.subparts.is_empty() {
        parsed_mail.get_body().unwrap()
    } else {
        parsed_mail.subparts.iter().filter_map(|part| {
            if part.ctype.mimetype == "text/plain" {
                part.get_body().ok()
            } else {
                None
            }
        }).collect::<Vec<String>>().join("\n")
    };
    let from = parsed_mail_headers.get_first_value("From").unwrap();
    let to = parsed_mail_headers.get_first_value("To").unwrap();
    let subject = parsed_mail_headers.get_first_value("Subject").unwrap();
    let date = parsed_mail_headers.get_first_value("Date").unwrap();
    let forwarded_body = format!(
        "---------- Forwarded message ---------\nFrom: {}\nTo: {}\nSent: {}\n\n{}",
        from, to, date, parsed_mail_body
    );
    let smtp_server = "smtp.gmail.com";
    let email = Message::builder()
        .from(sender.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(subject)
        .body(forwarded_body)
        .unwrap();
    let mailer = SmtpTransport::starttls_relay(smtp_server)
        .unwrap()
        .credentials(Credentials::new(sender.to_string(), smtp_password.to_string()))
        .hello_name(ClientId::Domain("localhost".to_string()))
        .build();
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Could not send email: {:?}", e),
    }
}
