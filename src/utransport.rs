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

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::oneshot;
use up_rust::{UCode, UListener, UMessage, UStatus, UTransport, UUri};

use crate::workers::{command::WorkerCommand, dispatcher::Iceoryx2WorkerDispatcher};

#[async_trait]
impl UTransport for Iceoryx2WorkerDispatcher {
    async fn send(&self, message: UMessage) -> Result<(), UStatus> {
        let (tx, rx) = oneshot::channel::<Result<(), UStatus>>();
        let command = WorkerCommand::Send {
            message,
            result_sender: tx,
        };
        self.command_sender
            .send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        rx.blocking_recv().map_err(|_| {
            UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed")
        })?
    }

    async fn register_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        up_rust::verify_filter_criteria(source_filter, sink_filter)?;
        let (tx, rx) = oneshot::channel::<Result<(), UStatus>>();
        let command = WorkerCommand::RegisterListener {
            source_filter: source_filter.clone(),
            sink_filter: sink_filter.cloned(),
            listener,
            result_sender: tx,
        };
        self.command_sender
            .send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        rx.blocking_recv().map_err(|_| {
            UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed")
        })?
    }

    async fn unregister_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        up_rust::verify_filter_criteria(source_filter, sink_filter)?;
        let (tx, rx) = oneshot::channel::<Result<(), UStatus>>();
        let command = WorkerCommand::UnregisterListener {
            source_filter: source_filter.clone(),
            sink_filter: sink_filter.cloned(),
            listener,
            result_sender: tx,
        };
        self.command_sender
            .send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        rx.blocking_recv().map_err(|_| {
            UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed")
        })?
    }
}
