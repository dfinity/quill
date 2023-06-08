#!/usr/bin/env bash
set -euxo pipefail

if [[ $# = 1 ]]; then # docker
    sudo() {
        "$@"
    }
fi

sudo apt-get update -y
sudo apt-get install libudev-dev:"${CROSS_DEB_ARCH:-amd64}" libssl-dev -y
