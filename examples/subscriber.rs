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
use tracing::info;
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_iceoryx2_rust::{MessagingPattern, transport::UTransportIceoryx2};

use crate::common::helpers::*;

mod common;

struct SubscriberListener(tokio::runtime::Runtime);

#[async_trait]
impl UListener for SubscriberListener {
    /// Spawns a task to process the received message. In this example, we simply print the message contents.
    async fn on_receive(&self, msg: UMessage) {
        self.0.spawn(async move {
            print_umessage(&msg);
        });
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("uProtocols UTransportIceoryx2 subscriber example");
    let source_filter =
        UUri::from_str("up://device1/10AB/3/80CD").expect("Failed to create source UUri");
    let transport = UTransportIceoryx2::build(MessagingPattern::PublishSubscribe)?;
    let ulistener = Arc::new(SubscriberListener(tokio::runtime::Runtime::new()?));
    transport
        .register_listener(&source_filter, None, ulistener)
        .await?;
    info!("Listener registered. Waiting for messages...");
    tokio::signal::ctrl_c().await.map_err(Box::from)
}
