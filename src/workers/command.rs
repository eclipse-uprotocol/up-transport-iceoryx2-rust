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
use tokio::sync::oneshot;
use up_rust::{UListener, UMessage, UStatus, UUri};

pub enum WorkerCommand {
    Send {
        message: UMessage,
        result_sender: oneshot::Sender<Result<(), UStatus>>,
    },
    RegisterListener {
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn UListener>,
        result_sender: oneshot::Sender<Result<(), UStatus>>,
    },
    UnregisterListener {
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn UListener>,
        result_sender: oneshot::Sender<Result<(), UStatus>>,
    },
}
