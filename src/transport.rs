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

use crate::workers::{command::WorkerCommand, dispatcher::Iceoryx2WorkerDispatcher};
use iceoryx2::node::{Node, NodeBuilder};
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use up_rust::UStatus;

pub struct UTransportIceoryx2<Service: iceoryx2::service::Service> {
    pub(crate) node: Node<Service>,
}

/// Acts as a uProtocol-specific interface for the Iceoryx2 transport system
impl<Service: iceoryx2::service::Service> UTransportIceoryx2<Service> {
    pub fn publish_subscribe() -> (Sender<WorkerCommand>, JoinHandle<Result<(), UStatus>>) {
        let (command_sender, worker_thread_handle) =
            Iceoryx2WorkerDispatcher::create_pubsub_worker(1024);
        (command_sender, worker_thread_handle)
    }

    pub(crate) fn default() -> Result<UTransportIceoryx2<Service>, UStatus> {
        let configure_fn: Option<fn(&NodeBuilder)> = None;
        Self::create(configure_fn)
    }

    fn create(
        configure: Option<impl FnOnce(&NodeBuilder)>,
    ) -> Result<UTransportIceoryx2<Service>, UStatus> {
        let node_builder = NodeBuilder::new();
        if let Some(configure) = configure {
            configure(&node_builder);
        }
        let node = node_builder
            .create::<Service>()
            .expect("Failed to create Iceoryx2 Node");
        Ok(UTransportIceoryx2 { node })
    }
}
