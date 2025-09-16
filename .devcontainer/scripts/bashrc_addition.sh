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

if [ -d "$HOME/.ssh" ]; then
    source ${WORKSPACE_FOLDER}/.devcontainer/scripts/ensure-ssh-permissions.sh
fi 
source ${WORKSPACE_FOLDER}/.devcontainer/scripts/colors.sh

PRIMARY_COLOR=$COLOR_GRAY
SECONDARY_COLOR=$COLOR_NC
INFO_COLOR=$COLOR_CYAN

WELCOME_INFO="${SECONDARY_COLOR}
██     ██ ███████ ██       ██████  ██████  ███    ███ ███████
██     ██ ██      ██      ██      ██    ██ ████  ████ ██     
██  █  ██ █████   ██      ██      ██    ██ ██ ████ ██ █████  
██ ███ ██ ██      ██      ██      ██    ██ ██  ██  ██ ██     
 ███ ███  ███████ ███████  ██████  ██████  ██      ██ ███████
${PRIMARY_COLOR}
to the Eclipse Foundation ${INFO_COLOR}$(basename "$WORKSPACE_FOLDER")${PRIMARY_COLOR} code repository

Workspace Folder       ${SECONDARY_COLOR}$WORKSPACE_FOLDER

${INFO_COLOR}Library Version(s)${PRIMARY_COLOR}
Rust Version           ${SECONDARY_COLOR}$(rustc --version)${PRIMARY_COLOR}
Cargo Version          ${SECONDARY_COLOR}$(cargo --version)${PRIMARY_COLOR}
Java Version           ${SECONDARY_COLOR}$(java --version)${PRIMARY_COLOR}

${INFO_COLOR}Git${PRIMARY_COLOR}
Username               ${SECONDARY_COLOR}$(git config user.name)${PRIMARY_COLOR}
Email                  ${SECONDARY_COLOR}$(git config user.email)${PRIMARY_COLOR}

to see this info again, use the ${INFO_COLOR}welcome${PRIMARY_COLOR} command${COLOR_NC}
"

alias welcome='printf "$WELCOME_INFO"'
welcome
