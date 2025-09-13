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

use std::str::FromStr;
use up_rust::{UMessage, UStatus, UUri};

pub struct Helpers {}

impl Helpers {
    pub fn create_uuris(source: &str, sink: Option<&str>) -> (UUri, Option<UUri>) {
        let source_uuri = UUri::from_str(source).expect("Failed to create source UUri");
        let sink_uuri = sink.map(|sink| UUri::from_str(sink).expect("Failed to create sink UUri"));
        (source_uuri, sink_uuri)
    }

    pub fn print_that_subscriber_service_has_been_registered(
        source_uuri: UUri,
        sink_filter_uuri: Option<UUri>,
    ) -> Result<(), UStatus> {
        let sink_str = match &sink_filter_uuri {
            Some(sink) => sink.to_uri(false),
            None => "".to_string(),
        };
        print!("Listener added with source '{source_uuri}' and sink '{sink_str}' uri's");
        Ok(())
    }

    /// Simply prints the [`UMessage`] instances source uri, sink uri, and payload to STDOUT
    pub fn print_umessage(msg: &UMessage) {
        let payload_utf8 = msg.payload.as_ref().map(|p| String::from_utf8_lossy(p));
        let (source_uri, sink_uri) = Helpers::get_source_and_sink_uri(msg);
        println!("Received a message!");
        println!("Source Uri: {source_uri:?}");
        println!("Sink Uri: {sink_uri:?}");
        println!("Payload: {payload_utf8:?}");
    }

    pub fn get_source_and_sink_uri(msg: &UMessage) -> (Option<String>, Option<String>) {
        let source_uri = msg
            .attributes
            .as_ref()
            .and_then(|a| a.source.as_ref())
            .map(|s| s.to_uri(false));
        let sink_uri = msg
            .attributes
            .as_ref()
            .and_then(|a| a.sink.as_ref())
            .map(|s| s.to_uri(false));
        (source_uri, sink_uri)
    }
}
