use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use pretty::{halt, notice, warn};
use system::{create_hash, truncate};

use std::time::Duration;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

use shared::{
    emails::Email,
    encrypt::Commands,
    errors::{AisError, UnifiedError},
};

#[derive(Debug)]
// #[allow(dead_code)]
struct TimedEmail {
    email: Email,
    received_at: Instant,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ErrorEmail {
    hash: String,
    subject: Option<String>,
    occoured_at: Instant,
}

#[allow(dead_code)]
fn send_email(subject: String, body: String) -> Result<(), UnifiedError> {
    // Build the email
    let email = Message::builder()
        .to("Enlightened One <enlightened@artisanhosting.net>"
            .parse()
            .map_err(|e| {
                UnifiedError::from_ais_error(AisError::new(&format!("Failed to build email: {}", e)))
            })?)
        .from(
            "ArtisanBot <ais_bot@artisanhosting.net>"
                .parse()
                .map_err(|e| {
                    UnifiedError::from_ais_error(AisError::new(&format!("Failed to build email: {}", e)))
                })?,
        )
        .subject(subject)
        .body(body)
        .map_err(|e| {
            UnifiedError::from_ais_error(AisError::new(&format!("Failed to build email: {}", e)))
        })?;

    // The smpt credentials
    let creds = Credentials::new(
        "ais_bot@artisanhosting.net".to_owned(),
        "&wvh\"x2)!62x93Cc-w".to_owned(), // This needed to be encrypted like the artisan.cf
    );

    let mailer = SmtpTransport::relay("mail.ramfield.net")
        .map_err(|e| {
            UnifiedError::from_ais_error(AisError::new(&format!(
                "Failed to connect to the mail server: {}",
                e
            )))
        })?
        .credentials(creds)
        .build();

    // Send the email
    mailer
        .send(&email)
        .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

    Ok(())
}

fn process_emails(emails: Arc<RwLock<Vec<TimedEmail>>>, errors: Arc<RwLock<Vec<ErrorEmail>>>) {
    loop {
        // Sleep for 1 minute
        thread::sleep(Duration::from_secs(60));

        // Lock the emails vector
        let mut email_errors = match errors.write() {
            Ok(vec) => vec,
            Err(_) => {
                eprintln!("Failed to acquire write lock on the error counter"); // Eventually add a uid and a phisical storage methode
                continue;
            }
        };

        // Lock the emails vector
        let mut email_vec = match emails.try_write() {
            Ok(vec) => vec,
            Err(_) => {
                eprintln!("Failed to acquire write lock on emails vector");
                email_errors.push(ErrorEmail {
                    hash: truncate(&create_hash("Failed to lock email array".to_owned()), 10)
                        .to_owned(),
                    subject: None,
                    occoured_at: Instant::now(),
                });
                continue;
            }
        };

        // Get the current time
        let current_time = Instant::now();

        // Iterate over emails in the vector
        let mut i = 0;
        let mut iteration_count = 0;
        let rate_limit = 7; // Set your desired rate limit here

        while i < email_vec.len() && iteration_count < rate_limit {
            if current_time.duration_since(email_vec[i].received_at) > Duration::from_secs(300) {
                println!("Expired email discarding: {:?}", email_vec[i]);
                email_vec.remove(i); // Remove expired email from the vector
            } else {
                match send_email(
                    email_vec[i].email.subject.to_owned(),
                    email_vec[i].email.body.to_owned(),
                ) {
                    Ok(_) => {
                        notice(&format!("Sending Email: {}-{}", &iteration_count.to_string(), &rate_limit));
                        email_vec.remove(i); // Remove sent email from the vector
                    }
                    Err(e) => {
                        eprintln!("An error occurred while sending email: {}", &e);
                        email_errors.push(ErrorEmail {
                            hash: truncate(&create_hash(e.to_string()), 10).to_owned(),
                            subject: Some(e.to_string()),
                            occoured_at: Instant::now(),
                        });
                        // Skip to the next email without removing the email from the vec i
                        i += 1;
                    }
                }
            }
            // Increment the iteration count
            iteration_count += 1;
        }
        match email_errors.len() < 1 {
            true => notice("No errors reported"),
            false => warn(&format!("Current errors: {}", email_errors.len())),
        }

        drop(email_errors);
        drop(email_vec);
    }
}

fn handle_client(
    mut stream: TcpStream,
    emails: Arc<RwLock<Vec<TimedEmail>>>,
) -> Result<(), UnifiedError> {
    let mut buffer = [0; 2048];
    let bytes_read = stream.read(&mut buffer).map_err(|e| {
        UnifiedError::from_ais_error(AisError::new(&format!("Failed to read buffered: {}", e)))
    })?;
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    notice("Emails recived");

    // Decrypt email data
    let decrypted_data = decrypt_received_data(&received_data)?;

    let email_data_plain = unsafe {
        String::from_utf8_unchecked(hex::decode(decrypted_data).map_err(|e| {
            UnifiedError::from_ais_error(AisError::new(&format!(
                "An error occoured while reading the hexed data: {}",
                &e.to_string()
            )))
        })?)
    };
    let email_data: Vec<&str> = email_data_plain.split("-=-").collect();
    let subject: &str = email_data[0];
    let body: &str = email_data[1];

    let email: Email = Email {
        subject: subject.to_owned(),
        body: body.to_owned(),
    };

    // Add email to the vector with current timestamp
    let timed_email: TimedEmail = TimedEmail {
        email: email.clone(),
        received_at: Instant::now(),
    };
    emails.try_write().unwrap().push(timed_email);
    drop(emails);

    // Send response to client
    stream.write_all(b"Email received").map_err(|e| {
        UnifiedError::from_ais_error(AisError::new(&format!("Error sending response: {}", e)))
    })?;
    stream.flush().map_err(|e| {
        UnifiedError::from_ais_error(AisError::new(&format!(
            "Error while flushing buffer: {}",
            e
        )))
    })?;

    Ok(())
}

fn decrypt_received_data(data: &str) -> Result<String, UnifiedError> {
    let decrypt = Commands::DecryptText(data.to_owned());
    let decrypted_data = decrypt.execute()?;
    Ok(decrypted_data.unwrap_or_else(|| "no data provided".to_owned()))
}

fn start_server(host: &str, port: u16, emails: Arc<RwLock<Vec<TimedEmail>>>) -> io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))?;
    println!("Server listening on {}:{}", host, port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let emails_clone = Arc::clone(&emails);
                thread::spawn(move || {
                    if let Err(err) = handle_client(stream, emails_clone) {
                        eprintln!("Error handling client: {}", err);
                    }
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }

    Ok(())
}

fn main() {
    let host = "0.0.0.0";
    let port = 1827;

    // Vector to store emails
    let emails: Arc<RwLock<Vec<TimedEmail>>> = Arc::new(RwLock::new(Vec::new()));
    let errors: Arc<RwLock<Vec<ErrorEmail>>> = Arc::new(RwLock::new(Vec::new()));

    // Start the email processing loop in a separate thread
    let emails_clone: Arc<RwLock<Vec<TimedEmail>>> = Arc::clone(&emails);
    let errors_clone: Arc<RwLock<Vec<ErrorEmail>>> = Arc::clone(&errors);
    thread::spawn(move || process_emails(emails_clone, errors_clone));

    // Start the server
    if let Err(err) = start_server(host, port, emails) {
        halt(&format!("Error starting server: {}", err));
    }
}
