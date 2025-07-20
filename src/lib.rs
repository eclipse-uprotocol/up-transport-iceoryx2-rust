use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UCode, UListener, UMessage, UStatus, UTransport, UUri};

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

#[allow(dead_code)]
impl Iceoryx2Transport {
    fn encode_uuri_segments(uuri: &UUri) -> Vec<String> {
        vec![
            uuri.authority_name.clone(),
            Self::encode_hex(uuri.uentity_type_id() as u32),
            Self::encode_hex(uuri.uentity_instance_id() as u32),
            Self::encode_hex(uuri.uentity_major_version() as u32),
            Self::encode_hex(uuri.resource_id() as u32),
        ]
    }

    fn encode_hex(value: u32) -> String {
        format!("{:X}", value)
    }

    fn compute_service_name(source: &UUri, sink: Option<&UUri>) -> Result<String, UStatus> {
        let join_segments = |segments: Vec<String>| segments.join("/");

        match (source, sink) {
            // RPC Request: source is a stub (0), sink is a valid method (1..=0x7FFF)
            (s, Some(sink)) if s.is_rpc_response() && sink.is_rpc_method() => {
                let segments = Self::encode_uuri_segments(sink);
                Ok(format!("up/{}", join_segments(segments)))
            }

            // Notification or RPC Response: sink is stub (0), source is valid (1..=0xFFFE)
            (source, Some(sink))
                if sink.is_rpc_response() && (1..=0xFFFE).contains(&source.resource_id) =>
            {
                let source_segments = Self::encode_uuri_segments(source);
                let sink_segments = Self::encode_uuri_segments(sink);
                Ok(format!(
                    "up/{}/{}",
                    join_segments(source_segments),
                    join_segments(sink_segments)
                ))
            }

            // Publish: source is valid (1..=0x7FFF), sink is None
            (source, None) if (1..=0x7FFF).contains(&source.resource_id) => {
                let segments = Self::encode_uuri_segments(source);
                Ok(format!("up/{}", join_segments(segments)))
            }

            // Invalid cases
            _ => Err(UStatus::fail_with_code(
                UCode::INVALID_ARGUMENT,
                "Unsupported or invalid UUri combination",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_uri(authority: &str, instance: u16, typ: u16, version: u8, resource: u16) -> UUri {
        let entity_id = ((instance as u32) << 16) | (typ as u32);
        UUri::try_from_parts(authority, entity_id, version, resource).unwrap()
    }

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_publish_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x7FFF);

        let name = Iceoryx2Transport::compute_service_name(&source, None).unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/7FFF");
    }

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_notification_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x80CD);
        let sink = test_uri("device1", 0x0000, 0x30EF, 0x04, 0x0000);
        let name = Iceoryx2Transport::compute_service_name(&source, Some(&sink)).unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/80CD/device1/30EF/0/4/0");
    }

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_rpc_request_service_name() {
        let sink = test_uri("device1", 0x0004, 0x03AB, 0x03, 0x0000);
        let reply_to = test_uri("device1", 0x0000, 0x00CD, 0x04, 0xB);

        let name = Iceoryx2Transport::compute_service_name(&sink, Some(&reply_to)).unwrap();
        assert_eq!(name, "up/device1/CD/0/4/B");
    }

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_rpc_response_service_name() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0xB);
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x0000);

        let name = Iceoryx2Transport::compute_service_name(&source, Some(&sink)).unwrap();
        assert_eq!(name, "up/device1/CD/0/4/B/device1/3AB/4/3/0");
    }

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_missing_uri_error() {
        let uuri = UUri::new();
        let result = Iceoryx2Transport::compute_service_name(&uuri, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().get_code(), UCode::INVALID_ARGUMENT);
    }
}
