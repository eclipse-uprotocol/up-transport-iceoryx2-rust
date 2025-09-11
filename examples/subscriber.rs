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
use iceoryx2::service::ipc;
use std::sync::Arc;
use tracing::info;
use up_rust::{UListener, UMessage};
use up_transport_iceoryx2_rust::{transport::UTransportIceoryx2, workers::command::WorkerCommand};

mod common;
use crate::common::helpers::Helpers;

struct SubscriberListener(tokio::runtime::Runtime);

#[async_trait]
impl UListener for SubscriberListener {
    /// Spawns a thread to process the received message. In this example, we simply print the message contents.
    async fn on_receive(&self, msg: UMessage) {
        self.0.spawn(async move {
            Helpers::print_umessage(&msg);
        });
    }
}

/// This example sets up a single threaded runtime for the UTransport implementation of the Iceoryx2 service to run on
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup the UMessage, subscriber, and command to register the subscriber on a dedicated thread
    // Iceoryx2 ipc::Service is used here, and does not support multi-threaded communication
    // See ipc_threadsafe::Service for that use case
    info!("uProtocols UTransportIceoryx2 subscriber example");
    // "//*/FFFFB1DA/1/8001"
    let (source_filter, sink_filter) = Helpers::create_uuris("up://device1/10AB/3/80CD", None);
    let (command_sender, _) = UTransportIceoryx2::<ipc::Service>::publish_subscribe();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("subscriber-example")
        .worker_threads(1)
        .build()?;
    let ulistener = Arc::new(SubscriberListener(runtime));

    let (result_sender, mut result_receiver) = tokio::sync::oneshot::channel();
    let register_listener_command = WorkerCommand::RegisterListener {
        source_filter: source_filter.clone(),
        sink_filter: sink_filter.clone(),
        listener: ulistener,
        result_sender,
    };
    command_sender.send(register_listener_command).await?;
    Helpers::print_that_subscriber_service_has_been_registered(source_filter, sink_filter)?;
    let result = result_receiver.try_recv()?;
    info!("Listener registration result: {result:?}");
    tokio::signal::ctrl_c().await.map_err(Box::from)
}
