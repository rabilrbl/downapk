#!/usr/bin/env bash

dpkg --add-architecture $CROSS_DEB_ARCH
apt-get update && apt-get install --assume-yes libssl-dev:$CROSS_DEB_ARCH
