#!/usr/bin/env bash

set -e

DIR_BAK=~/.cargo/bak
DIR_BIN=~/.cargo/bin

PAC=${1}
VER=${2}
BIN=$(echo -n "${PAC}" | sed 's/-cli//g')

mkdir -p ${DIR_BAK}

if [[ -f "${DIR_BAK}/${BIN}" ]]; then
  echo "Using cached ${PAC}"
  cp -f "${DIR_BAK}/${BIN}" "${DIR_BIN}/${BIN}"
fi

echo "Installing ${PAC}"
cargo install --version ${VER} ${PAC}
cp -f "${DIR_BIN}/${BIN}" "${DIR_BAK}/${BIN}"
