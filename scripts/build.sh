#!/bin/bash
set -ex

cd "${0%/*}/.."

wasm-pack build --target web
wasm-opt -Oz -o ./pkg/line_diff_wasm_bg.wasm ./pkg/line_diff_wasm_bg.wasm
