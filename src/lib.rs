use async_trait::async_trait;
use iceoryx2::prelude::*;
use protobuf::MessageField;
use std::sync::Arc;
use up_rust::{UAttributes, UCode, UMessage, UStatus, UTransport, UAttributesValidators};

mod custom_header;
pub use custom_header::CustomHeader;

mod raw_bytes;
use raw_bytes::RawBytes;

use std::collections::HashMap;
use std::thread;

enum TransportCommand {
    Send {
        message: UMessage,
        response: std::sync::mpsc::Sender<Result<(), UStatus>>,
    },
}

pub struct Iceoryx2Transport {
    command_sender: std::sync::mpsc::Sender<TransportCommand>,
}

impl Iceoryx2Transport {
    pub fn new() -> Result<Self, UStatus> {
        let (tx, rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            Self::background_task(rx);
        });

        Ok(Self { command_sender: tx })
    }

    fn encode_uuri_segments(uuri: &up_rust::UUri) -> Vec<String> {
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

    fn handle_send(
        publisher: &iceoryx2::port::publisher::Publisher<ipc::Service, RawBytes, CustomHeader>,
        message: UMessage,
    ) -> Result<(), UStatus> {
        let payload_bytes = message.payload.clone().unwrap_or_default().to_vec();
        let raw_payload = RawBytes::from_bytes(&payload_bytes);
        let header = CustomHeader::from_message(&message)?;

        let sample = publisher.loan_uninit().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to loan sample: {e}"))
        })?;

        let mut sample_final = sample.write_payload(raw_payload);
        *sample_final.user_header_mut() = header;

        sample_final.send().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to send: {e}"))
        })?;

        Ok(())
    }

    fn background_task(rx: std::sync::mpsc::Receiver<TransportCommand>) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async {
            let node = match NodeBuilder::new().create::<ipc::Service>() {
                Ok(node) => node,
                Err(e) => {
                    eprintln!("Failed to create iceoryx2 node: {}", e);
                    return;
                }
            };

            let mut publishers: HashMap<
                String,
                iceoryx2::port::publisher::Publisher<ipc::Service, RawBytes, CustomHeader>,
            > = HashMap::new();

            loop {
                while let Ok(command) = rx.try_recv() {
                    match command {
                        TransportCommand::Send { message, response } => {
                            let service_name = match Self::compute_service_name(&message) {
                                Ok(name) => name,
                                Err(e) => {
                                    let _ = response.send(Err(e));
                                    continue;
                                }
                            };

                            let publisher =
                                publishers.entry(service_name.clone()).or_insert_with(|| {
                                    let service_name_res: Result<ServiceName, _> =
                                        service_name.as_str().try_into();
                                    let service = node
                                        .service_builder(&service_name_res.unwrap())
                                        .publish_subscribe::<RawBytes>()
                                        .user_header::<CustomHeader>()
                                        .open_or_create()
                                        .expect("Failed to create service");

                                    service
                                        .publisher_builder()
                                        .create()
                                        .expect("Failed to create publisher")
                                });

                            let result = Self::handle_send(publisher, message);
                            let _ = response.send(result);
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
    }
}

#[async_trait]
impl UTransport for Iceoryx2Transport {
    async fn send(&self, message: UMessage) -> Result<(), UStatus> {
        let attributes = message.attributes.as_ref().ok_or_else(|| {
            UStatus::fail_with_code(UCode::INVALID_ARGUMENT, "missing message attributes")
        })?;

        let validator = UAttributesValidators::get_validator_for_attributes(attributes);
        if let Err(e) = validator.validate(attributes) {
            return Err(UStatus::fail_with_code(UCode::INVALID_ARGUMENT, &format!("Invalid message attributes: {}", e),
        ));
        }

        let (tx, rx) = std::sync::mpsc::channel();

        let command = TransportCommand::Send {
            message,
            response: tx,
        };

        self.command_sender
            .send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;

        rx.recv().map_err(|_| {
            UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed")
        })?
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use up_rust::{UMessageBuilder, UPayloadFormat, UUID, UUri};

    fn test_uri(authority: &str, instance: u16, typ: u16, version: u8, resource: u16) -> UUri {
        let entity_id = ((instance as u32) << 16) | (typ as u32);
        UUri::try_from_parts(authority, entity_id, version, resource).unwrap()
    }

    #[tokio::test]
    async fn test_send_publish_success() {
        let transport = Iceoryx2Transport::new().unwrap();
        let source = test_uri("device", 0x1000, 0xABCD, 0x01, 0x8000);
        let message = UMessageBuilder::publish(source)
            .build_with_payload(vec![1, 2, 3], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let result = transport.send(message).await;
        assert!(result.is_ok(), "Expected successful publish send");
    }

    #[tokio::test]
    async fn test_send_request_success() {
        let transport = Iceoryx2Transport::new().unwrap();
        let sink = test_uri("device", 0x1111, 0xCDCD, 0x02, 0x0001);
        let reply_to = test_uri("device", 0x2222, 0xEEEE, 0x01, 0x0000);
        let message = UMessageBuilder::request(sink, reply_to, 1234)
            .build_with_payload(vec![9, 8, 7], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let result = transport.send(message).await;
        assert!(result.is_ok(), "Expected successful request send");
    }

    #[tokio::test]
    async fn test_send_notification_success() {
        let transport = Iceoryx2Transport::new().unwrap();
        let source = test_uri("device", 0xAAAA, 0x1111, 0x01, 0x1000);
        let sink = test_uri("device", 0xBBBB, 0x2222, 0x01, 0x0000);
        let message = UMessageBuilder::notification(source, sink)
            .build_with_payload(vec![0], UPayloadFormat::UPAYLOAD_FORMAT_RAW)
            .unwrap();

        let result = transport.send(message).await;
        assert!(result.is_ok(), "Expected successful notification send");
    }
}
