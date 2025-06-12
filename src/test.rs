// src/test.rs

use super::*;
use bytes::Bytes;
use std::sync::Arc;

use protobuf::MessageField;  // if you use this in tests

// Example synchronous test (no async needed) -works
#[test]
fn test_custom_header_from_user_header() {
    let header = CustomHeader { version: 2, timestamp: 1000 };
    let new_header = CustomHeader::from_user_header(&header).unwrap();
    assert_eq!(new_header.version, 2);
    assert_eq!(new_header.timestamp, 1000);
}

// Async test requires Tokio runtime - work
#[tokio::test]
async fn test_transport_creation() {
    let transport = Iceoryx2Transport::new();
    assert!(transport.is_ok());
}

//checks that umessage is correctly parsed into transmission data - works
#[test]
fn test_transmission_data_from_message() {
    let data = TransmissionData {
        x: 42,
        y: -7,
        funky: 3.14,
    };

    let bytes = data.to_bytes();
    let mut msg = UMessage::new();
    msg.payload = Some(Bytes::from(bytes));

    let result = TransmissionData::from_message(&msg);
    assert!(result.is_ok());
    let decoded = result.unwrap();
    assert_eq!(decoded.x, 42);
    assert_eq!(decoded.y, -7);
    assert!((decoded.funky - 3.14).abs() < 1e-6);
}

