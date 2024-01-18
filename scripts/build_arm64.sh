#!/usr/bin/env bash

apt-get install libssl-dev -y
OPENSSL_PC_DIR=$(find /usr/ -name "openssl.pc")
export PKG_CONFIG_PATH=$OPENSSL_PC_DIR:$PKG_CONFIG_PATH
