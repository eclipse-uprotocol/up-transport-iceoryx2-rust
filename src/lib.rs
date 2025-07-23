use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UCode, UListener, UMessage, UStatus, UTransport, UUri};

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

    fn determine_message_type(source: &UUri, sink: Option<&UUri>) -> Result<MessageType, UStatus> {
        match (source.resource_id, sink.map(|s| s.resource_id)) {
            (0, Some(sink_id)) if (1..=0x7FFF).contains(&sink_id) => Ok(MessageType::RpcRequest),
            (src_id, Some(0)) if (1..=0xFFFE).contains(&src_id) => Ok(MessageType::RpcResponseOrNotification),
            (src_id, _) if (1..=0x7FFF).contains(&src_id) => Ok(MessageType::Publish),
            _ => Err(UStatus::fail_with_code(
                UCode::INVALID_ARGUMENT,
                "Unsupported UMessageType",
            )),
        }
    }    

    fn compute_service_name(source: &UUri, sink: Option<&UUri>) -> Result<String, UStatus> {
        let join_segments = |segments: Vec<String>| segments.join("/");
    
        match Self::determine_message_type(source, sink)? {
            MessageType::RpcRequest => {
                if let Some(sink_uri) = sink {
                    let segments = Self::encode_uuri_segments(sink_uri);
                    Ok(format!("up/{}", join_segments(segments)))
                } else {
                    Err(UStatus::invalid_argument("sink required for RpcRequest"))
                }
            }
            MessageType::RpcResponseOrNotification => {
                if let Some(sink_uri) = sink {
                    let source_segments = Self::encode_uuri_segments(source);
                    let sink_segments = Self::encode_uuri_segments(sink_uri);
                    Ok(format!(
                        "up/{}/{}",
                        join_segments(source_segments),
                        join_segments(sink_segments)
                    ))
                } else {
                    Err(UStatus::invalid_argument("sink required for RpcResponseOrNotification"))
                }
            }
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

    // performing successful tests for service name computation

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

    // performing failing tests for service name computation

    #[test]
    // .specitem[dsn~up-attributes-request-source~1]
    // .specitem[dsn~up-attributes-response-source~1]
    // .specitem[dsn~up-attributes-notification-source~1]
    fn test_missing_uri_error() {
        let uuri = UUri::new();
        let result = Iceoryx2Transport::compute_service_name(&uuri, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().get_code(), UCode::INVALID_ARGUMENT);
    }

    #[test]
    //both source and sink have resource ID equal to 0
    // .specitem[dsn~up-attributes-request-source~1]
    // .specitem[dsn~up-attributes-request-sink~1]
    // .specitem[dsn~up-attributes-response-source~1]
    // .specitem[dsn~up-attributes-response-sink~1]
    fn test_fail_resource_id_error() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0x000);
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x0000);
        let result = Iceoryx2Transport::compute_service_name(&source,Some(&sink));
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }
    
    #[test]
    //source has resource id=0 but missing sink
    // .specitem[dsn~up-attributes-request-sink~1]
    // .specitem[dsn~up-attributes-request-source~1]
    fn test_fail_missing_sink_error() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0x000);
        let result = Iceoryx2Transport::compute_service_name(&source,None);
       assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }
    
    #[test]
    //missing source URI
    // .specitem[dsn~up-attributes-request-source~1]
    // .specitem[dsn~up-attributes-response-source~1]
    // .specitem[dsn~up-attributes-notification-source~1]
    fn test_fail_missing_source_error() {
        let uuri = UUri::new();
        let sink = test_uri("device1", 0x0004, 0x3AB, 0x3, 0x000);
        let result = Iceoryx2Transport::compute_service_name(&uuri,Some(&sink));
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }
}
