#!/usr/bin/env bash
set -euxo pipefail

if [[ $# = 1 ]]; then # docker
    dpkg --add-architecture "$CROSS_DEB_ARCH"
    if [[ $1 != "arm-"* && $1 != *"-musl" ]]; then
        arch=":$CROSS_DEB_ARCH"
    fi
    sudo() {
        "$@"
    }
fi

sudo apt-get update -y
sudo apt-get install "libudev-dev${arch-}" -y
