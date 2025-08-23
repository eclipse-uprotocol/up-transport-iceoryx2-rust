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

#region sudo stuff

HOST_OS=""
ARCH_AND_OS_FUNC="${WORKSPACE_FOLDER}/.devcontainer/scripts/functions/get-arch-and-os.sh"
if [[ -f "$ARCH_AND_OS_FUNC" ]]; then
    source $ARCH_AND_OS_FUNC
    read -r _ HOST_OS <<< "$(get_arch_and_os)"
fi
if [[ "$HOST_OS" == "darwin" ]]; then # darwin == Mac OS
    # This is a workaround which is againt necessary on MacOS 14.0, it looks like this bug is back:
    # https://github.com/microsoft/vscode-dev-containers/issues/1487#issuecomment-1143907307
    # grant permissions to mounted rust volume
    echo "(Mac OS only) Granting permissions to mounted rust volume"
    sudo chown vscode:vscode /rust-volume

    # create /.cargo/config.toml in root folder
    sudo mkdir /.cargo/
    sudo touch /.cargo/config.toml
    sudo bash -c "cat << EOF > /.cargo/config.toml
    [build]
    target-dir = \"/rust-volume/target\"
    EOF"
fi

if ! grep -q "bashrc_addition" ~/.bashrc && [[ -f "${WORKSPACE_FOLDER}/.devcontainer/scripts/bashrc_addition.sh" ]]; then
    echo "source ${WORKSPACE_FOLDER}/.devcontainer/scripts/bashrc_addition.sh" >> ~/.bashrc
fi

#endregion

for arg in "$@"; do
    if [[ "$arg" == --workspace-folder=* ]]; then
        WORKSPACE_FOLDER="${arg#--workspace-folder=}"
    fi
done
INSTALL_RUST_TOOLCHAIN_FUNC="${WORKSPACE_FOLDER}/.devcontainer/scripts/functions/install-rust-toolchain.sh"
if [[ -f "$INSTALL_RUST_TOOLCHAIN_FUNC" ]]; then
    source $INSTALL_RUST_TOOLCHAIN_FUNC
    install_rust_toolchain "$@"
fi
