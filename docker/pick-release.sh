#!/usr/bin/env sh

set -eux

if uname -m | grep x86_64; then
    TARGET=x86_64-unknown-linux-gnu
elif uname -m | grep aarch64 ; then
    TARGET=armv7-unknown-linux-gnueabihf
else
    echo "No binary for architecture $(uname -m)"
    exit 1
fi

cp ${RELEASE_DIR}/spis-server-${TARGET} ${BIN_LOCATION}