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

get_arch_and_os() {
    ARCH=$(uname -m)
    OS=$(uname -s | tr "[:upper:]" "[:lower:]")

    case "$ARCH" in
    x86_64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64 | arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac

    case "$OS" in
    linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    *)
        echo "OS is unsupported or not implemented in this script: $OS"
        exit 1
        ;;
    esac
    echo "$TARGET_ARCH $TARGET_OS"
}
