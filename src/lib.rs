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

use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::{ServiceName, ZeroCopySend},
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};
use tokio::sync::RwLock;
use up_rust::ComparableListener;

use crate::{umessage::UMessageZeroCopy, uprotocolheader::UProtocolHeader};

pub(crate) mod service_name_mapping;
pub mod transport;
pub(crate) mod umessage;
pub(crate) mod uprotocolheader;
pub(crate) mod utransport_pubsub;
pub(crate) mod workers;

pub use iceoryx2::prelude::MessagingPattern;

pub trait BaseUserHeader: Debug + ZeroCopySend {}
pub trait BasePayload: Debug + ZeroCopySend {}

pub(crate) type PublisherSet<Service> =
    RwLock<HashMap<ServiceName, Arc<Publisher<Service, UMessageZeroCopy, UProtocolHeader>>>>;
pub(crate) type SubscriberSet<Service> =
    RwLock<HashMap<ServiceName, Arc<Subscriber<Service, UMessageZeroCopy, UProtocolHeader>>>>;
pub(crate) type ListenerMap = RwLock<HashMap<ServiceName, HashSet<ComparableListener>>>;
