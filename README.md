# HL7 PDF Sender

A Rust-based utility for sending HL7 messages over TCP/IP with HL7 MDM^T02 message type.

## Features

- HL7 v2.5 message formatting
- TCP/IP transmission with timeout handling
- File size validation (max 10MB)
- Proper HL7 control characters
- Configurable network settings

## Installation

1. Clone the repository:

```bash
git clone https://github.com/bkutasi/hl7sender
cd hl7sender
```
2. Run tests:
```bash	
cargo  test
```

3. Build the project:

```bash
cargo build --release
```

## Usage

```bash
./target/release/hl7sender --host <host> --port <port> --message <message> --timeout <timeout>
```

### Arguments:
- `--host`: The target HL7 server hostname or IP address
- `--port`: The port number of the HL7 server
- `--message`: Path to the PDF file to be sent
- `--timeout`: The maximum number of seconds to wait for a response

### Example:

```bash
./target/release/hl7sender --host localhost --port 2525 --message message.hl7 --timeout 60
```


## Message Format

The program takes HL7 messages with the following structure:
- Message type: MDM^T02
- Base64 encoded PDF in OBX segment
- ASCII character encoding
- Standard HL7 v2.5 delimiters

## Error Handling

The program includes error handling for:
- File size limitations (10MB max)
- Network timeouts (30 seconds)
- Connection failures
- Invalid responses

## Technical Details

The implementation consists of a main function which send the prepared HL7 message:
- `send_hl7_message`: Handles TCP/IP communication

For detailed implementation, see the source code:

[src/main.rs](src/main.rs)

Testing coverage:

```
2024-12-01T11:58:09.827358Z  INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| src/main.rs: 65, 97, 106, 110-114
|| Tested/Total Lines:
|| src/main.rs: 35/43 +0.00%
|| 
81.40% coverage, 35/43 lines covered, +0.00% change in coverage
```
