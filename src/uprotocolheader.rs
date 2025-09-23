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

use iceoryx2::prelude::ZeroCopySend;
use iceoryx2_bb_container::vec::FixedSizeVec;

pub const MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH: usize = 1000;

/// Also see [uAttributes Mapping to iceoryx2 user header](https://github.com/eclipse-uprotocol/up-spec/blob/0cc43c8afb7d7cbd3169ffe093be761c57308cef/up-l1/iceoryx2.adoc#411-uattributes-mapping-to-iceoryx2-user-header)
#[repr(C)]
#[derive(ZeroCopySend, Debug, Default)]
pub struct UProtocolHeader {
    pub(crate) uprotocol_major_version: u8,
    pub(crate) uattributes_serialized: FixedSizeVec<u8, MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH>,
}
