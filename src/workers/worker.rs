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

use crate::utransport_pubsub::Iceoryx2PubSub;

pub struct Iceoryx2Worker {
    pub keep_alive: Arc<AtomicBool>,
    pub transport: Arc<Iceoryx2PubSub>,
}

impl Iceoryx2Worker {
    pub fn new(transport: Arc<Iceoryx2PubSub>) -> Self {
        Self {
            keep_alive: Arc::new(AtomicBool::new(true)),
            transport,
        }
    }
}
