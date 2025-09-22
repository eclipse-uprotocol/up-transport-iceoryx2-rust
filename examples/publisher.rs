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

use std::{error::Error, str::FromStr};
use up_rust::{UMessage, UMessageBuilder, UPayloadFormat, UTransport, UUri};
use up_transport_iceoryx2_rust::{MessagingPattern, transport::UTransportIceoryx2};

mod common;
use crate::common::*;

fn create_umessage(source_filter: &UUri, counter: u64) -> Result<UMessage, Box<dyn Error>> {
    let payload = format!("Hello, Iceoryx2! Message {counter}");
    let umessage = UMessageBuilder::publish(source_filter.clone())
        .build_with_payload(payload.into_bytes(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)?;
    Ok(umessage)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("uProtocol UTransportIceoryx2 publisher example");
    let source_filter = UUri::from_str(SOURCE_FILTER_STR).expect("Failed to create source UUri");
    let transport = UTransportIceoryx2::build(MessagingPattern::PublishSubscribe)?;
    let mut counter: u64 = 0;
    let cycle_time_str = format!("{CYCLE_TIME:?}");
    println!(
        "Publishing a UMessage with an incrementing counter every '{cycle_time_str}' second with source filter '{SOURCE_FILTER_STR}'"
    );
    loop {
        counter += 1;
        let umessage = create_umessage(&source_filter, counter)?;
        let memory_address = &umessage;
        println!(
            "Publishing message with source filter '{SOURCE_FILTER_STR}' and memory address {memory_address:p}"
        );
        transport.send(umessage).await?;
        tokio::time::sleep(CYCLE_TIME).await;
    }
}
