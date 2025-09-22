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

use crate::utransport_pubsub::Iceoryx2PubSub;
use iceoryx2::prelude::MessagingPattern;
use up_rust::{UCode, UStatus, UTransport};

pub struct UTransportIceoryx2 {}

/// Acts as a uProtocol-specific interface for the Iceoryx2 transport system
impl UTransportIceoryx2 {
    pub fn build(messaging_pattern: MessagingPattern) -> Result<Arc<impl UTransport>, UStatus> {
        match messaging_pattern {
            MessagingPattern::PublishSubscribe => Ok(UTransportIceoryx2::build_publish_subscribe()),
            _ => Err(UStatus::fail_with_code(
                UCode::UNIMPLEMENTED,
                "Unimplemented messaging pattern",
            )),
        }
    }

    fn build_publish_subscribe() -> Arc<Iceoryx2PubSub> {
        Iceoryx2PubSub::new()
    }
}
