// Copyright (c) 2023 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT


/*
use iceoryx2::prelude::*;
use up_rust::{UCode, UMessage, UStatus};

#[derive(Debug, Clone, Copy, ZeroCopySend, Default)]
// optional type name; if not set, `core::any::type_name::<TransmissionData>()` is used
#[type_name("TransmissionData")]
#[repr(C)]
pub struct TransmissionData {
    pub x: i32,
    pub y: i32,
    pub funky: f64,
}

impl TransmissionData {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, UStatus> {
        if bytes.len() != std::mem::size_of::<Self>() {
            return Err(UStatus::fail_with_code(
                UCode::INVALID_ARGUMENT,
                "Invalid byte length for TransmissionData",
            ));
        }

        let mut data = Self {
            x: 0,
            y: 0,
            funky: 0.0,
        };
        let ptr = &mut data as *mut Self as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        }
        Ok(data)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; std::mem::size_of::<Self>()];
        let ptr = self as *const Self as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), bytes.len());
        }
        bytes
    }

    pub fn from_message(message: &UMessage) -> Result<Self, UStatus> {
        let payload = message.payload.clone().unwrap_or_default();
        Self::from_bytes(payload.to_vec())
    }
}

*/