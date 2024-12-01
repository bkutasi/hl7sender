#[cfg(test)]
mod tests;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use clap::Parser;

const DEFAULT_TIMEOUT: u64 = 30;
const BUFFER_SIZE: usize = 4096;

struct Config {
    timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT),
        }
    }
}

fn send_hl7_message_with_config(host: &str, port: u16, message: &str, config: Config) -> io::Result<String> {
    let mut stream = TcpStream::connect((host, port))?;
    
    // Set timeouts
    stream.set_read_timeout(Some(config.timeout))?;
    stream.set_write_timeout(Some(config.timeout))?;
    
    // Prepare message with MLLP frame
    let framed_message = format!("\x0B{}\x1C\x0D", message);
    
    // Write message
    stream.write_all(framed_message.as_bytes())?;
    stream.flush()?;
    
    // Read response with larger buffer
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; BUFFER_SIZE];
    
    loop {
        match stream.read(&mut temp_buffer) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&temp_buffer[..n]);
                if buffer.ends_with(b"\x1C\x0D") {
                    break;
                }
            },
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(e),
        }
    }
    
    if buffer.is_empty() {
        return Err(io::Error::new(io::ErrorKind::TimedOut, "Read timed out"));
    }
    
    // Remove MLLP frame
    let response = buffer.strip_prefix(b"\x0B").unwrap_or(&buffer);
    let response = response.strip_suffix(b"\x1C\x0D").unwrap_or(response);
    
    String::from_utf8(response.to_vec())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Host address of the HL7 server
    #[arg(short, long, default_value = "localhost")]
    host: String,

    /// Port number of the HL7 server
    #[arg(short, long)]
    port: u16,

    /// Path to the HL7 message file
    #[arg(short, long)]
    message: String,

    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,
}

fn run(args: Args) -> Result<(), String> {
    let config = Config {
        timeout: Duration::from_secs(args.timeout),
    };

    let mut file = File::open(&args.message)
        .map_err(|e| format!("Failed to open message file: {}", e))?;
    let mut message = String::new();
    file.read_to_string(&mut message)
        .map_err(|e| format!("Failed to read message file: {}", e))?;

    match send_hl7_message_with_config(&args.host, args.port, &message, config) {
        Ok(response) => {
            println!("HL7 Message Sent");
            println!("Response from server:");
            println!("{}", response);
            Ok(())
        }
        Err(e) => Err(format!("Failed to send HL7 message: {}", e)),
    }
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}