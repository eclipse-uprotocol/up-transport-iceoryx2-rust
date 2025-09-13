// ################################################################################
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https: //www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
// ################################################################################

use iceoryx2::prelude::{MessagingPattern, ServiceName};
use up_rust::{UCode, UMessageType, UStatus, UUri};

pub struct ServiceNameMapper;

/// uProtocol [`UUri`] to Iceoryx2 [`ServiceName`] mapping(s)
impl ServiceNameMapper {
    fn encode_uuri_segments(uuri: &UUri) -> Vec<String> {
        vec![
            Self::get_authority_name(uuri),
            Self::encode_hex(uuri.uentity_type_id() as u32),
            Self::encode_hex(uuri.uentity_instance_id() as u32),
            Self::encode_hex(uuri.uentity_major_version() as u32),
            Self::encode_hex(uuri.resource_id() as u32),
        ]
    }

    fn encode_hex(value: u32) -> String {
        format!("{value:X}")
    }

    fn get_authority_name(source_uuri: &UUri) -> String {
        if source_uuri.authority_name.is_empty() {
            match hostname::get().unwrap().into_string() {
                Ok(hostname) => hostname,
                Err(_) => "unknown".to_string(),
            }
        } else {
            source_uuri.authority_name.clone()
        }
    }

    fn determine_message_type(
        source: &UUri,
        _sink: Option<&UUri>,
        messaging_pattern: MessagingPattern,
    ) -> Result<UMessageType, UStatus> {
        // let src_id = source.resource_id;
        // let sink_id = sink.map(|s| s.resource_id);

        if Self::is_a_publish(source, messaging_pattern) {
            return Ok(UMessageType::UMESSAGE_TYPE_PUBLISH);
        }

        Err(UStatus::fail_with_code(
            UCode::INVALID_ARGUMENT,
            "Could not determine a valid UMessageType from the provided UUri(s)",
        ))
    }

    fn is_a_publish(source: &UUri, messaging_pattern: MessagingPattern) -> bool {
        source.is_empty() == false && messaging_pattern == MessagingPattern::PublishSubscribe
    }

    pub fn compute_service_name(
        source: &UUri,
        sink: Option<&UUri>,
        messaging_pattern: MessagingPattern,
    ) -> Result<ServiceName, UStatus> {
        let join_segments = |segments: Vec<String>| segments.join("/");
        let message_type = Self::determine_message_type(source, sink, messaging_pattern)?;
        let service_name_str = match message_type {
            UMessageType::UMESSAGE_TYPE_REQUEST => {
                let Some(sink_uri) = sink else {
                    return Err(UStatus::fail_with_code(
                        UCode::INVALID_ARGUMENT,
                        format!(
                            "sink required for UMessageType {:?}",
                            UMessageType::UMESSAGE_TYPE_REQUEST
                        ),
                    ));
                };
                let segments = Self::encode_uuri_segments(sink_uri);
                format!("up/{}", join_segments(segments))
            }
            UMessageType::UMESSAGE_TYPE_RESPONSE | UMessageType::UMESSAGE_TYPE_NOTIFICATION => {
                let Some(sink_uri) = sink else {
                    return Err(UStatus::fail_with_code(
                        UCode::INVALID_ARGUMENT,
                        format!(
                            "sink required for UMessageType {:?} or {:?}",
                            UMessageType::UMESSAGE_TYPE_RESPONSE,
                            UMessageType::UMESSAGE_TYPE_NOTIFICATION
                        ),
                    ));
                };
                let source_segments = Self::encode_uuri_segments(source);
                let sink_segments = Self::encode_uuri_segments(sink_uri);
                format!(
                    "up/{}/{}",
                    join_segments(source_segments),
                    join_segments(sink_segments)
                )
            }
            UMessageType::UMESSAGE_TYPE_PUBLISH => {
                let segments = Self::encode_uuri_segments(source);
                format!("up/{}", join_segments(segments))
            }
            _ => {
                return Err(UStatus::fail_with_code(
                    UCode::INVALID_ARGUMENT,
                    "Unsupported UMessageType for service name computation",
                ));
            }
        };
        Ok(ServiceName::new(service_name_str.as_str()).expect("Failed to create service name"))
    }
}

#[cfg(test)]
mod tests {
    use crate::service_name_mapping::ServiceNameMapper;
    use iceoryx2::prelude::MessagingPattern;
    use up_rust::{UCode, UUri};

    fn test_uri(authority: &str, instance: u16, typ: u16, version: u8, resource: u16) -> UUri {
        let entity_id = ((instance as u32) << 16) | (typ as u32);
        UUri::try_from_parts(authority, entity_id, version, resource).unwrap()
    }

    // performing successful tests for service name computation

    #[test]
    // [specitem,oft-sid="dsn~up-transport-iceoryx2-service-name~1",oft-needs="utest"]
    fn test_publish_service_name() {
        let source = test_uri("device1", 0x0000, 0x10AB, 0x03, 0x7FFF);

        let name = ServiceNameMapper::compute_service_name(
            &source,
            None,
            MessagingPattern::PublishSubscribe,
        )
        .unwrap();
        assert_eq!(name, "up/device1/10AB/0/3/7FFF");
    }

    // performing failing tests for service name computation

    #[test]
    // .specitem[dsn~up-attributes-request-source~1]
    // .specitem[dsn~up-attributes-response-source~1]
    // .specitem[dsn~up-attributes-notification-source~1]
    fn test_missing_uri_error() {
        let uuri = UUri::new();
        let result = ServiceNameMapper::compute_service_name(
            &uuri,
            None,
            MessagingPattern::PublishSubscribe,
        );

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
        let result = ServiceNameMapper::compute_service_name(
            &source,
            Some(&sink),
            MessagingPattern::PublishSubscribe,
        );
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }

    #[test]
    //source has resource id=0 but missing sink
    // .specitem[dsn~up-attributes-request-sink~1]
    // .specitem[dsn~up-attributes-request-source~1]
    fn test_fail_missing_sink_error() {
        let source = test_uri("device1", 0x0000, 0x00CD, 0x04, 0x000);
        let result = ServiceNameMapper::compute_service_name(
            &source,
            None,
            MessagingPattern::RequestResponse,
        );
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
        let result = ServiceNameMapper::compute_service_name(
            &uuri,
            Some(&sink),
            MessagingPattern::PublishSubscribe,
        );
        assert!(result.is_err_and(|err| err.get_code() == UCode::INVALID_ARGUMENT));
    }
}
