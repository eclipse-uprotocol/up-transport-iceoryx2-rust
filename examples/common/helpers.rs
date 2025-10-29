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

use up_rust::UMessage;

/// Simply prints the [`UMessage`] instances source uri, sink uri, and payload to STDOUT
#[allow(dead_code)]
pub fn print_umessage(msg: &UMessage) {
    let payload_utf8 = msg.payload.as_ref().map(|p| String::from_utf8_lossy(p));
    let (source_uri, sink_uri) = get_source_and_sink_uri(msg);
    println!("Source Uri: {source_uri:?}");
    println!("Sink Uri: {sink_uri:?}");
    println!("Payload: {payload_utf8:?}");
}

fn get_source_and_sink_uri(msg: &UMessage) -> (Option<String>, Option<String>) {
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
