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

use std::sync::{Arc, atomic::Ordering};
use tokio::runtime::Runtime;
use up_rust::UStatus;

use crate::{utransport_pubsub::Iceoryx2PubSub, workers::worker::Iceoryx2Worker};

pub struct Iceoryx2WorkerDispatcher {}

impl Iceoryx2WorkerDispatcher {
    pub fn start_listener_worker(runtime: &Runtime, transport: Arc<Iceoryx2PubSub>) {
        let worker = Iceoryx2Worker::new(transport.clone());
        let future = Iceoryx2WorkerDispatcher::run(worker);
        runtime.spawn(future);
    }

    async fn run(worker: Iceoryx2Worker) -> Result<(), UStatus> {
        while worker.keep_alive.load(Ordering::Relaxed) {
            worker.transport.relay().await?;
        }
        Ok(())
    }
}
