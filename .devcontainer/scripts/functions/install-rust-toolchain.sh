#!/bin/bash
################################################################################
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https: //www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
################################################################################

install_rust_toolchain() {
    source ${WORKSPACE_FOLDER}/.devcontainer/scripts/functions/get-arch-and-os.sh
    read -r TARGET_ARCH TARGET_OS <<< "$(get_arch_and_os)"
    ARCH_AND_OS="${TARGET_ARCH}-${TARGET_OS}"
    echo "Detected architecture and OS: $ARCH_AND_OS"
    RUST_TOOLCHAIN="stable-$ARCH_AND_OS"
    if rustup toolchain list | grep -q $RUST_TOOLCHAIN; then
        echo "Rust toolchain '$RUST_TOOLCHAIN' is already installed."
        return 0
    fi
    echo "Adding rustup target '$ARCH_AND_OS'"
    rustup target add "$ARCH_AND_OS"
    echo "Installing Rust toolchain for '$ARCH_AND_OS'"
    rustup toolchain install "$RUST_TOOLCHAIN"
}
