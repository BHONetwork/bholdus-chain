#!/usr/bin/env bash

set -xe

RUSTC_VERSION=1.56.1;
PACKAGE=$PACKAGE;

SRTOOL_TAG=$RUSTC_VERSION srtool build -p $PACKAGE