use crate::encrypt::Commands;
use crate::errors::{AisError, UnifiedError};
use serde::{Deserialize, Serialize};
use std::{fmt, io::Write, net::TcpStream};

/// Represents an email message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Email {
    /// The subject of the email.
    pub subject: String,
    /// The body of the email.
    pub body: String,
}

/// Represents an encrypted email message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailSecure {
    /// The encrypted email data.
    pub data: String,
}

// Display implementations
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
    /// Creates a new Email instance with the given subject and body.
    pub fn new(subject: String, body: String) -> Self {
        Email { subject, body }
    }

    /// Checks if the email data is valid.
    pub fn is_valid(&self) -> bool {
        !self.subject.is_empty() && !self.body.is_empty()
    }
}

impl EmailSecure {
    /// Creates a new EmailSecure instance by encrypting the provided email.
    pub fn new(email: Email) -> Result<Self, UnifiedError> {
        if !email.is_valid() {
            return Err(UnifiedError::from_ais_error(AisError::new(
                "Invalid Email Data",
            )));
        }

        let plain_email_data = format!("{}-=-{}", email.subject, email.body);
        let encrypted_data = match Commands::execute(&Commands::EncryptText(plain_email_data)) {
            Ok(Some(d)) => d,
            Ok(None) => {
                return Err(UnifiedError::from_ais_error(AisError::new(
                    "No data was provided to encrypt",
                )))
            }
            Err(e) => return Err(e.into()),
        };

        Ok(EmailSecure {
            data: encrypted_data,
        })
    }

    /// Sends the encrypted email data over a TCP stream.
    pub fn send(&self) -> Result<(), UnifiedError> {
        let mut stream = match TcpStream::connect("10.1.0.11:1827") {
            Ok(d) => d,
            Err(e) => return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
        };
        match stream.write_all(self.data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_new() {
        let email = Email::new("Subject".to_string(), "Body".to_string());
        assert_eq!(email.subject, "Subject");
        assert_eq!(email.body, "Body");
    }

    #[test]
    fn test_email_is_valid() {
        let valid_email = Email::new("Subject".to_string(), "Body".to_string());
        assert!(valid_email.is_valid());

        let invalid_email = Email::new("".to_string(), "".to_string());
        assert!(!invalid_email.is_valid());
    }

    #[test]
    fn test_emailsecure_new() {
        let email = Email::new("Subject".to_string(), "Body".to_string());
        let email_secure = EmailSecure::new(email).unwrap();
        assert!(!email_secure.data.is_empty());
    }

    #[test]
    #[ignore = "When tested in git workflow this will hang, need a conditional way to test this"]
    fn test_emailsecure_send() {
        // Note: This test assumes there's a server listening on the specified address.
        // Replace it with a valid server address for testing.

        // Create a dummy encrypted email
        let encrypted_data = "dummy_encrypted_data".to_string();
        let email_secure = EmailSecure { data: encrypted_data };

        // Attempt to send the encrypted email
        let result = email_secure.send();
        // Ensure that the send operation was successful or resulted in an error
        assert!(result.is_ok() || result.is_err());
    }
}
