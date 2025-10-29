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

use async_trait::async_trait;
use std::{str::FromStr, sync::Arc};
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_iceoryx2_rust::{MessagingPattern, transport::UTransportIceoryx2};

mod common;
use crate::common::*;

struct ConsolePrinter;

#[async_trait]
impl UListener for ConsolePrinter {
    async fn on_receive(&self, message: UMessage) {
        let payload_memory_address = message.payload.as_ref().unwrap();
        println!("Received a message!");
        print_umessage(&message);
        println!("Payload Memory address: {payload_memory_address:p}");
        println!();
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("uProtocol UTransportIceoryx2 subscriber example");
    let source_filter = UUri::from_str(SOURCE_FILTER_STR).expect("Failed to create source UUri");
    let transport = UTransportIceoryx2::build(MessagingPattern::PublishSubscribe)?;
    let ulistener = Arc::new(ConsolePrinter);
    transport
        .register_listener(&source_filter, None, ulistener)
        .await?;
    println!(
        "Listening to message from source filter '{SOURCE_FILTER_STR}'. Press CTRL+C to kill this subscriber"
    );
    println!("Waiting for messages...");
    tokio::signal::ctrl_c().await.map_err(Box::from)
}
