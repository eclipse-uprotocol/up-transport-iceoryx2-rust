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

use std::fmt::Debug;
use iceoryx2::prelude::ZeroCopySend;
use up_rust::UMessage;

#[derive(Debug)]
pub struct UMessageZeroCopy(pub UMessage);

unsafe impl ZeroCopySend for UMessageZeroCopy {
    unsafe fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}
