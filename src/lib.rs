use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri, UCode};

/// This will be the main struct for our uProtocol transport.
/// It will hold the state necessary to communicate with iceoryx2,
/// such as the service connection and active listeners.
pub struct Iceoryx2Transport {}

// The #[async_trait] attribute enables async functions in our trait impl.
#[async_trait]
impl UTransport for Iceoryx2Transport {
    async fn send(&self, _message: UMessage) -> Result<(), UStatus> {
        todo!();
    }

    async fn register_listener(
        &self,
        _source_filter: &UUri,
        _sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        todo!()
    }

    async fn unregister_listener(
        &self,
        _source_filter: &UUri,
        _sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        todo!()
    }
}

impl Iceoryx2Transport {
    fn encode_uuri_segments(uuri: &UUri) -> Vec<String> {
        vec![
            uuri.authority_name.clone(),
            Self::encode_hex_no_leading_zeros(uuri.uentity_type_id() as u32),
            Self::encode_hex_no_leading_zeros(uuri.uentity_instance_id() as u32),
            Self::encode_hex_no_leading_zeros(uuri.uentity_major_version() as u32),
            Self::encode_hex_no_leading_zeros(uuri.resource_id() as u32),
        ]
    }

    fn encode_hex_no_leading_zeros(value: u32) -> String {
        format!("{:X}", value)
    }

    fn compute_service_name(message: &UMessage) -> Result<String, UStatus> {
        let join_segments = |segments: Vec<String>| segments.join("/");

        if message.is_publish() {
            let source = message.source().ok_or_else(|| {
                UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "Missing source URI")
            })?;
            let segments = Self::encode_uuri_segments(source);
            Ok(format!("up/{}", join_segments(segments)))
        } else if message.is_request() {
            let sink = message.sink().ok_or_else(|| {
                UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "Missing sink URI")
            })?;
            let segments = Self::encode_uuri_segments(sink);
            Ok(format!("up/{}", join_segments(segments)))
        } else if message.is_response() || message.is_notification() {
            let source = message.source().ok_or_else(|| {
                UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "Missing source URI")
            })?;
            let sink = message.sink().ok_or_else(|| {
                UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "Missing sink URI")
            })?;

            let source_segments = Self::encode_uuri_segments(source);
            let sink_segments = Self::encode_uuri_segments(sink);
            Ok(format!(
                "up/{}/{}",
                join_segments(source_segments),
                join_segments(sink_segments)
            ))
        } else {
            Err(UStatus::fail_with_code(
                UCode::INVALID_ARGUMENT,
                "Unsupported UMessageType",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use up_rust::{UMessageBuilder, UPayloadFormat};

    fn dummy_uuid() -> up_rust::UUID {
        up_rust::UUID::build()
    }

    fn test_uri(authority: &str, instance: u16, typ: u16, version: u8, resource: u16) -> UUri {
        let entity_id = ((instance as u32) << 16) | (typ as u32);
        UUri::try_from_parts(authority, entity_id, version, resource).unwrap()
    }

    #[test]
    fn test_publish_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x80CD);
        let message = UMessageBuilder::publish(source.clone())
            .build_with_payload(vec![], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let name = Iceoryx2Transport::compute_service_name(&message).unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/80CD");
    }

    #[test]
    fn test_notification_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x80CD);
        let sink = test_uri("device1", 0x0000, 0x30EF, 0x04, 0x0000);
        let message = UMessageBuilder::notification(source.clone(), sink.clone())
            .build_with_payload(vec![], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let name = Iceoryx2Transport::compute_service_name(&message).unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/80CD/device1/30EF/0/4/0");
    }

    #[test]
    fn test_rpc_request_service_name() {
        let sink = test_uri("device1", 0x0000, 0x00CD, 0x04, 0x000B);
        let reply_to = test_uri("device1", 0x0000, 0x0001, 0x01, 0x0000);

        let message = UMessageBuilder::request(sink.clone(), reply_to, 1000)
            .build_with_payload(vec![], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let name = Iceoryx2Transport::compute_service_name(&message).unwrap();
        assert_eq!(name, "up/device1/CD/0/4/B");
    }

    #[test]
    fn test_rpc_response_service_name() {
        let source = test_uri("device1", 0x0001, 0x0020, 0x01, 0x0000);
        let sink = test_uri("device1", 0x0001, 0x00CD, 0x04, 0x1000);
        let uuid = dummy_uuid();

        let message = UMessageBuilder::response(source.clone(), uuid, sink.clone())
            .build_with_payload(vec![], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let name = Iceoryx2Transport::compute_service_name(&message).unwrap();
        assert_eq!(name, "up/device1/CD/1/4/1000/device1/20/1/1/0");
    }

    #[test]
    fn test_missing_uri_error() {
        let message = UMessage::new();
        let result = Iceoryx2Transport::compute_service_name(&message);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().get_code(), UCode::INVALID_ARGUMENT);
    }
}
