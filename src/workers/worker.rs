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

use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::{Mutex, mpsc::Receiver};
use up_rust::{UListener, UMessage, UStatus, UUri};

use crate::workers::command::WorkerCommand;

pub(crate) trait Iceoryx2Worker {
    // begin UTransport wrapper methods

    fn send(&mut self, message: UMessage) -> Result<(), UStatus>;

    fn register_listener(
        &mut self,
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;

    fn unregister_listener(
        &mut self,
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus>;

    // end UTransport wrapper methods

    fn get_command_receiver(&self) -> Arc<Mutex<Receiver<WorkerCommand>>>;

    fn keep_alive(&self) -> &AtomicBool;

    async fn receive_and_notify_listeners(&self) -> Result<(), UStatus>;
}
