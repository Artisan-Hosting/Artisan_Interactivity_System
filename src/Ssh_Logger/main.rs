use std::net::UdpSocket;

fn parse_syslog_message(message: &str) -> Option<(String, String, String)> {
    // Example parsing logic for SSH connection data
    // Customize this function based on your syslog message format
    let parts: Vec<&str> = message.split_whitespace().collect();
    if parts.len() >= 10 && parts[4] == "sshd" {
        let remote_ip = parts[7].to_string();
        let duration = parts[9].to_string();
        let user = parts[10].to_string();
        Some((remote_ip, duration, user))
    } else {
        None
    }
}

fn syslog_receiver(host: &str, port: u16) {
    // Create a UDP socket
    let socket = UdpSocket::bind(format!("{}:{}", host, port)).expect("Failed to bind socket");

    println!("Syslog receiver listening on {}:{}", host, port);

    // Buffer to store incoming data
    let mut buf = [0; 1024];

    loop {
        // Receive incoming syslog messages
        let (num_bytes, _src_addr) = socket.recv_from(&mut buf).expect("Failed to receive data");

        // Parse syslog message to extract SSH connection data
        let message = std::str::from_utf8(&buf[..num_bytes]).expect("Failed to parse message");
        if let Some((remote_ip, duration, user)) = parse_syslog_message(message) {
            // Print extracted SSH connection data
            println!("Remote IP: {}, Duration: {}, User: {}", remote_ip, duration, user);
        }
    }
}

fn main() {
    // Define the host and port to listen on
    let host = "0.0.0.0";  // Listen on all available interfaces
    let port = 1514;         // Default syslog port

    // Start the syslog receiver
    syslog_receiver(host, port);
}