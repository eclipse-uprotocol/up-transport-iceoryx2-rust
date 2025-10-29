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

use std::time::Duration;

/// The source filter publisher examples will use
pub static SOURCE_FILTER_STR: &str = "up://device1/10AB/3/80CD";

/// A UMessage will be published at this frequency
#[allow(dead_code)]
pub static CYCLE_TIME: Duration = Duration::from_secs(1);
