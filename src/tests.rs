use super::*;
use std::fs::write;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};
use std::io::{self, Write, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// Mock TCP server for testing purposes
struct MockTcpServer {
    listener: TcpListener,
}

impl MockTcpServer {
    /// Creates a new MockTcpServer bound to a random available port
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        Self { listener }
    }

    /// Returns the port number the server is listening on
    fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    /// Handles an incoming connection by sending a predefined response
    fn handle_connection(mut stream: TcpStream, response: &str) {
        let mut buffer = [0; 1024];
        // Read incoming data (simulated)
        let _ = stream.read(&mut buffer);
        // Write the response back to the client
        stream.write_all(response.as_bytes()).unwrap();
    }
}

struct TestMsg {
    path: PathBuf,
    _dir: TempDir,
}

fn create_test_message(content: &str) -> io::Result<TestMsg> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test.hl7");
    write(&file_path, content)?;
    Ok(TestMsg {
        path: file_path,
        _dir: dir,
    })
}

#[test]
fn test_send_hl7_message_success() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((stream, _)) = server.listener.accept() {
            MockTcpServer::handle_connection(stream, "ACK");
        }
    });

    let args = Args {
        host: "localhost".to_string(),
        port,
        message: "test.hl7".to_string(),
        timeout: 30,
    };

    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";

    let result = send_hl7_message_with_config(&args.host, args.port, message, Config::default());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("ACK"));
}

#[test]
fn test_send_hl7_message_connection_refused() {
    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let result = send_hl7_message_with_config("localhost", 9999, message, Config::default());
    assert!(result.is_err());
}

#[test]
fn test_send_hl7_message_would_block() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((_stream, _)) = server.listener.accept() {
            // Do not send any data to cause WouldBlock
        }
    });

    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let result = send_hl7_message_with_config("127.0.0.1", port, message, Config::default());
    assert!(result.is_err());
    let kind = result.unwrap_err().kind();
    assert!(
        kind == io::ErrorKind::TimedOut
            || kind == io::ErrorKind::WouldBlock
            || kind == io::ErrorKind::ConnectionReset
    );
}

#[test]
fn test_send_hl7_message_invalid_utf8() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = server.listener.accept() {
            // Send invalid UTF-8 bytes
            let invalid_bytes = vec![0xff, 0xfe, 0xfd];
            stream.write_all(&invalid_bytes).unwrap();
        }
    });

    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let result = send_hl7_message_with_config("127.0.0.1", port, message, Config::default());
    assert!(result.is_err());
    let kind = result.unwrap_err().kind();
    assert!(
        kind == io::ErrorKind::InvalidData
            || kind == io::ErrorKind::ConnectionReset
    );
}

#[test]
fn test_send_hl7_message_timeout() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((_stream, _)) = server.listener.accept() {
            // Do not respond to trigger timeout
        }
    });

    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let result = send_hl7_message_with_config("127.0.0.1", port, message, Config::default());
    assert!(result.is_err());
    let kind = result.unwrap_err().kind();
    assert!(
        kind == io::ErrorKind::TimedOut
            || kind == io::ErrorKind::ConnectionReset
    );
}

#[test]
fn test_run_invalid_arguments() {
    let args = Args {
        host: "".to_string(),
        port: 0,
        message: "".to_string(),
        timeout: 0,
    };
    let result = run(args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to open message file"));
}

#[test]
fn test_run_send_hl7_failure() {
    let args = Args {
        host: "localhost".to_string(),
        port: 9999,
        message: "/nonexistent/path/message.hl7".to_string(),
        timeout: 30,
    };
    let result = run(args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to open message file"));
}

#[test]
fn test_run_send_hl7_success() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((stream, _)) = server.listener.accept() {
            MockTcpServer::handle_connection(stream, "ACK");
        }
    });

    let message_content = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                          OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let test_msg = create_test_message(message_content).unwrap();

    let args = Args {
        host: "localhost".to_string(),
        port,
        message: test_msg.path.to_str().unwrap().to_string(),
        timeout: 30,
    };
    let result = run(args);
    assert!(result.is_ok());
}

#[test]
fn test_send_hl7_message_would_block_mapping() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = server.listener.accept() {
            // Read the incoming message but don't respond
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer);
            // Sleep to ensure timeout
            thread::sleep(Duration::from_secs(31));
        }
    });

    let message = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                  OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let result = send_hl7_message_with_config("127.0.0.1", port, message, Config::default());
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.kind() == io::ErrorKind::TimedOut);
}

#[test]
fn test_send_hl7_message_custom_timeout() {
    // Test constants
    const LONG_TIMEOUT_SECS: u64 = 3;
    const SHORT_TIMEOUT_MILLIS: u64 = 100;
    const SERVER_DELAY_SECS: u64 = 2;
    const TEST_MESSAGE: &str = "\
        MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
        OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";

    // First test case: Successful response within timeout
    {
        let server = MockTcpServer::new();
        let port = server.port();

        // Spawn server that responds with ACK after a short delay
        thread::spawn(move || {
            if let Ok((mut stream, _)) = server.listener.accept() {
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer);
                thread::sleep(Duration::from_secs(1)); // Delay shorter than timeout
                stream.write_all(b"ACK").unwrap();
            }
        });

        // Test with long timeout - should succeed
        let config = Config {
            timeout: Duration::from_secs(LONG_TIMEOUT_SECS),
        };
        let result = send_hl7_message_with_config("127.0.0.1", port, TEST_MESSAGE, config);
        assert!(result.is_ok(), "Expected successful message delivery with {LONG_TIMEOUT_SECS}s timeout");
        assert!(result.unwrap().contains("ACK"), "Expected ACK response from server");
    }

    // Second test case: Timeout before response
    {
        let server = MockTcpServer::new();
        let port = server.port();

        // Spawn server that delays longer than the timeout
        thread::spawn(move || {
            if let Ok((mut stream, _)) = server.listener.accept() {
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer);
                thread::sleep(Duration::from_secs(SERVER_DELAY_SECS)); // Delay longer than timeout
                let _ = stream.write_all(b"ACK"); // Write should not succeed due to timeout
            }
        });

        // Test with short timeout - should fail with timeout
        let config = Config {
            timeout: Duration::from_millis(SHORT_TIMEOUT_MILLIS),
        };
        let result = send_hl7_message_with_config("127.0.0.1", port, TEST_MESSAGE, config);
        
        assert!(result.is_err(), "Expected timeout error with {}ms timeout", SHORT_TIMEOUT_MILLIS);
        let error = result.unwrap_err();
        // Debug output, uncomment to see error kind
        // eprintln!("Received error kind: {:?}", error.kind()); 
        
        // Check for expected timeout-related errors
        assert!(
            matches!(error.kind(),
                io::ErrorKind::TimedOut |      // Standard timeout
                io::ErrorKind::WouldBlock |    // Non-blocking operation would block
                io::ErrorKind::ConnectionReset | // Connection reset by peer
                io::ErrorKind::UnexpectedEof    // Connection closed unexpectedly
            ),
            "Expected timeout-related error, got: {:?}", error.kind()
        );
    }
}

#[test]
fn test_run_with_custom_timeout() {
    let server = MockTcpServer::new();
    let port = server.port();

    thread::spawn(move || {
        if let Ok((stream, _)) = server.listener.accept() {
            MockTcpServer::handle_connection(stream, "ACK");
        }
    });

    let message_content = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|SendingFac|202401011230||MDM^T02|MSG123|P|2.5|||||ASCII\r\
                          OBX|1|ED|PDF^Application^PDF^Base64||dGVzdCBwZGY=|||||F\r\x1c\r";
    let test_msg = create_test_message(message_content).unwrap();

    let args = Args {
        host: "localhost".to_string(),
        port,
        message: test_msg.path.to_str().unwrap().to_string(),
        timeout: 45,
    };

    let config = Config {
        timeout: Duration::from_secs(args.timeout),
    };

    let result = send_hl7_message_with_config(&args.host, args.port, &message_content, config);
    assert!(result.is_ok());
}
