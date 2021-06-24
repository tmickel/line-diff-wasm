#!/bin/bash
set -ex

cd "${0%/*}/.."

./scripts/build.sh
wasm-pack publish
