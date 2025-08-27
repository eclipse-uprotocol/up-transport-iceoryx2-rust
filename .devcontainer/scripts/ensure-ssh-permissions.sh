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

# Sometimes when mounting volumes, the files unexpectedly change their read-write-execute status.
# This is dependent on the OS the Docker container is mounting from, and how it exposes its
# files permissions, ownership, and "executability" to the container and the containers user.
# 
# More about specific permissions inside the .ssh folder 
# https://serverfault.com/a/253314
sudo find ~/.ssh -maxdepth 1 -type f -exec chmod 600 {} \;
chmod u+r ~/.ssh/config
chmod u+r ~/.ssh/known_hosts
