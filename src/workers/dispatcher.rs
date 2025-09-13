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

use iceoryx2::prelude::MessagingPattern;
use std::sync::{Arc, atomic::Ordering};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};
use up_rust::{UCode, UStatus};

use crate::{
    transport::UTransportIceoryx2,
    workers::{
        command::WorkerCommand, pubsub_worker::Iceoryx2PubSubWorker, worker::Iceoryx2Worker,
    },
};

pub struct Iceoryx2WorkerDispatcher {
    pub command_sender: std::sync::mpsc::Sender<WorkerCommand>,
    pub messaging_pattern: MessagingPattern,
    pub handle: JoinHandle<Result<(), UStatus>>,
}

impl Iceoryx2WorkerDispatcher {
    pub fn create_pubsub_worker(
        buffer_size: usize,
    ) -> (Sender<WorkerCommand>, JoinHandle<Result<(), UStatus>>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<WorkerCommand>(buffer_size);
        let threadsafe_rx = Arc::new(Mutex::new(rx));
        let handle = tokio::spawn(async {
            let future = Iceoryx2WorkerDispatcher::run(threadsafe_rx);

            let current_thread_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    UStatus::fail_with_code(
                        UCode::INTERNAL,
                        format!("Failed to build current_thread runtime: {e}"),
                    )
                })?;
            current_thread_runtime.block_on(future)
        });
        (tx, handle)
    }

    async fn run(rx: Arc<Mutex<Receiver<WorkerCommand>>>) -> Result<(), UStatus> {
        // non-threadsafe state goes here
        // any iceoryx2 transports that use the `ipc::Service` service implementation is not threadsafe
        let transport = UTransportIceoryx2::default()?;
        let mut worker =
            Iceoryx2PubSubWorker::new(rx, transport, MessagingPattern::PublishSubscribe);
        //
        while worker.keep_alive().load(Ordering::Relaxed) {
            let command_receiver = worker.get_command_receiver().clone();
            let mut command_receiver = command_receiver.lock().await;
            if let Ok(command) = command_receiver.try_recv() {
                Iceoryx2WorkerDispatcher::process_command(&mut worker, command)?;
            }
            worker.receive_and_notify_listeners().await?;
            // .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, e.to_string()))?;
        }
        Ok(())
    }

    fn process_command<Worker: Iceoryx2Worker>(
        worker: &mut Worker,
        command: WorkerCommand,
    ) -> Result<(), UStatus> {
        let channel_response = match command {
            WorkerCommand::Send {
                message,
                result_sender,
            } => {
                let result = worker.send(message);
                result_sender.send(result)
            }
            WorkerCommand::RegisterListener {
                source_filter,
                sink_filter,
                listener,
                result_sender,
            } => {
                let result = worker.register_listener(source_filter, sink_filter, listener);
                result_sender.send(result)
            }
            WorkerCommand::UnregisterListener {
                source_filter,
                sink_filter,
                listener,
                result_sender,
            } => {
                let result = worker.unregister_listener(source_filter, sink_filter, listener);
                result_sender.send(result)
            }
        };
        match channel_response {
            Err(_) => Err(UStatus::fail_with_code(
                UCode::INTERNAL,
                "Failed to send result of command through the response channel",
            )),
            Ok(_) => Ok(()),
        }
    }
}
