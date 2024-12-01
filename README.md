# HL7 PDF Sender

A Rust-based utility for sending PDF documents as HL7 messages over TCP/IP. This tool encodes PDF files in Base64 format and transmits them using the HL7 MDM^T02 message type.

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

2. Build the project:

```bash
cargo build --release
```

## Usage

```bash
./target/release/hl7sender <host> <port> <pdf_path>
```

### Arguments:
- `host`: The target HL7 server hostname or IP address
- `port`: The port number of the HL7 server
- `pdf_path`: Path to the PDF file to be sent

### Example:

```bash
./target/release/hl7sender localhost 2575 message.hl7
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
2024-11-30T16:32:13.805516Z  INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| src/main.rs: 56-57, 83-89, 91, 98-102
|| Tested/Total Lines:
|| src/main.rs: 38/53 +16.80%
|| 
71.70% coverage, 38/53 lines covered, +16.80% change in coverage
```
