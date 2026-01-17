#!/bin/sh
set -eu

cargo build

codesign --force --deep --sign - target/debug/lensql

exec target/debug/lensql
