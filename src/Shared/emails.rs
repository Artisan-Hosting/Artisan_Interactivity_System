use std::{fmt, io::Write, net::TcpStream};
use serde::{Deserialize, Serialize};
use crate::errors::{AisError, UnifiedError};
use crate::encrypt::Commands;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Email {
    pub subject: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailSecure {
    pub data: String,
}

// Displays
impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.subject, self.body)
    }
}

impl fmt::Display for EmailSecure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Email {
    pub fn new(subject: String, body: String) -> Self {
        Email { subject, body }
    }

    pub fn is_valid(&self) -> bool {
        !self.subject.is_empty() && !self.body.is_empty()
    }
}

impl EmailSecure {
    pub fn new(email: Email) -> Result<Self, UnifiedError> {
        if !email.is_valid() {
            return Err(UnifiedError::AisError(AisError::new("Invalid email data")));
        }

        let plain_email_data = format!("{}-=-{}", email.subject, email.body);
        let encrypted_data = match Commands::execute(&Commands::EncryptText(plain_email_data)) {
            Ok(Some(d)) => d,
            Ok(None) => return Err(UnifiedError::AisError(AisError::new("No data provided for encryption"))),
            Err(e) => return Err(e.into()),
        };

        Ok(EmailSecure { data: encrypted_data })
    }

    pub fn send(&self) -> Result<(), UnifiedError> {
        let mut stream = match TcpStream::connect("10.1.0.11:1827"){
            Ok(d) => d,
            Err(e) => return Err(UnifiedError::AisError(AisError::new(&e.to_string()))),
        };
        match stream.write_all(self.data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(UnifiedError::AisError(AisError::new(&e.to_string()))),
        }
    }
}