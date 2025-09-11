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

const MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH: usize = 1000; // choosing ~1000 u8s
// somewhat arbitrarily
// this should be confirmed

#[repr(C)]
#[derive(ZeroCopySend, Debug)]
pub struct UProtocolHeader {
    uprotocol_major_version: u8,
    uattributes_serialized: FixedSizeVec<u8, MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH>,
}
