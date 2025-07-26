use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri, UCode};

/// This will be the main struct for our uProtocol transport.
/// It will hold the state necessary to communicate with iceoryx2,
/// such as the service connection and active listeners.
pub struct Iceoryx2Transport {}

enum MessageType {
    RpcRequest,
    RpcResponseOrNotification,
    Publish,
}

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

    /// Assumption: valid source and sink URIs provided:
    /// send() makes use of UAttributesValidator
    /// register_listener() and unregister_listener() use verify_filter_criteria()
    /// Criteria for identification of message types can be found here: https://github.com/eclipse-uprotocol/up-spec/blob/main/basics/uattributes.adoc
    fn determine_message_type(source: &UUri, sink: Option<&UUri>) -> Result<MessageType, UStatus> {
        let src_id = source.resource_id;
        let sink_id = sink.map(|s| s.resource_id);

        if src_id == 0 {
            if let Some(id) = sink_id {
                if id >= 1 && id <= 0x7FFF {
                    return Ok(MessageType::RpcRequest);
                }
            }
        } else if sink_id == Some(0) && src_id >= 1 && src_id <= 0xFFFE {
            return Ok(MessageType::RpcResponseOrNotification);
        } else if src_id >= 1 && src_id <= 0x7FFF {
            return Ok(MessageType::Publish);
        }

        Err(UStatus::fail_with_code(
            UCode::INVALID_ARGUMENT,
            "Unsupported UMessageType",
        ))
    }
    /// Called by send(), register_listener() and unregister_listener()
    fn compute_service_name(source: &UUri, sink: Option<&UUri>) -> Result<String, UStatus> {
        let join_segments = |segments: Vec<String>| segments.join("/");
        // [impl->dsn~up-transport-iceoryx2-service-name~1]
        match Self::determine_message_type(source, sink)? {
            // [impl->dsn~up-transport-iceoryx2-service-name~1]
            MessageType::RpcRequest => {
                let Some(sink_uri) = sink else {
                    return Err(UStatus::fail_with_code(
                        UCode::INVALID_ARGUMENT,
                        "sink required for RpcRequest",
                    ));
                };
                let segments = Self::encode_uuri_segments(sink_uri);
                Ok(format!("up/{}", join_segments(segments)))
            }
            // [impl->dsn~up-transport-iceoryx2-service-name~1]
            MessageType::RpcResponseOrNotification => {
                let Some(sink_uri) = sink else {
                    return Err(UStatus::fail_with_code(
                        UCode::INVALID_ARGUMENT,
                        "sink required for ResponseOrNotification",
                    ));
                };
                let source_segments = Self::encode_uuri_segments(source);
                let sink_segments = Self::encode_uuri_segments(sink_uri);
                Ok(format!(
                    "up/{}/{}",
                    join_segments(source_segments),
                    join_segments(sink_segments)
                ))
            }
            // [impl->dsn~up-transport-iceoryx2-service-name~1]
            MessageType::Publish => {
                let segments = Self::encode_uuri_segments(source);
                Ok(format!("up/{}", join_segments(segments)))
            }
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
    // [utest->dsn~up-transport-iceoryx2-service-name~1]
    fn test_publish_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x7FFF);

        let name = Iceoryx2Transport::compute_service_name(&source,None).unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/7FFF");
    }

    #[test]

    // [utest->dsn~up-transport-iceoryx2-service-name~1]
    fn test_notification_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x80CD);
        let sink = test_uri("device1", 0x0000, 0x30EF, 0x04, 0x0000);
        let name = Iceoryx2Transport::compute_service_name(&source,Some(&sink)).unwrap();

        assert_eq!(name, "up/device1/10AB/0/3/80CD/device1/30EF/0/4/0");
    }

    #[test]

    // [utest->dsn~up-transport-iceoryx2-service-name~1]
    fn test_rpc_request_service_name() {
        let sink = test_uri("device1", 0x0004, 0x03AB, 0x03, 0x0000);
        let reply_to = test_uri("device1", 0x0000, 0x00CD, 0x04, 0xB);
        let name = Iceoryx2Transport::compute_service_name(&sink, Some(&reply_to)).unwrap();
      
        assert_eq!(name, "up/device1/CD/0/4/B");
    }

    #[test]
    // [utest->dsn~up-transport-iceoryx2-service-name~1]
    fn test_rpc_response_service_name() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0xB);
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x0000);
        let name = Iceoryx2Transport::compute_service_name(&source,Some(&sink)).unwrap();
      
        assert_eq!(name, "up/device1/CD/0/4/B/device1/3AB/4/3/0");
    }

    #[test]
    // [utest->dsn~up-transport-iceoryx2-service-name~1]
    fn test_missing_uri_error() {
        let uuri = UUri::new();
        let result = Iceoryx2Transport::compute_service_name(&uuri, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().get_code(), UCode::INVALID_ARGUMENT);
    }

    #[test]
    // [utest->dsn~up-transport-iceoryx2-service-name~1]

    fn test_fail_resource_id_error() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0x000);
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x0000);
        let result = Iceoryx2Transport::compute_service_name(&source, Some(&sink));
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }

    #[test]
    // [utest->dsn~up-transport-iceoryx2-service-name~1]

    fn test_fail_missing_source_error() {
        let uuri = UUri::new();
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x000);
        let result = Iceoryx2Transport::compute_service_name(&uuri, Some(&sink));
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }
}

