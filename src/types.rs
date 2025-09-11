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
    service::ipc,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};
use up_rust::ComparableListener;

use crate::{umessage::UMessageZeroCopy, uprotocolheader::UProtocolHeader};

pub trait BaseUserHeader: Debug + ZeroCopySend {}
pub trait BasePayload: Debug + ZeroCopySend {}

pub(crate) type PublisherSet =
    HashMap<ServiceName, Publisher<ipc::Service, UMessageZeroCopy, UProtocolHeader>>;
pub(crate) type SubscriberSet =
    HashMap<ServiceName, Subscriber<ipc::Service, UMessageZeroCopy, UProtocolHeader>>;
pub(crate) type ListenerMap = HashMap<ServiceName, HashSet<ComparableListener>>;
