#!/bin/bash

set -e

TARGET=""

if [ "$1" == "debug" ]; then
  TARGET="debug"
elif [ "$1" == "release" ]; then
  TARGET="release"
else
  echo "Unrecognized target \`$1\`"
  exit 1
fi

if [ "${TARGET}" == "release" ]; then
  cargo build --target wasm32-unknown-unknown --release
else
  cargo build --target wasm32-unknown-unknown
fi

wasm-bindgen --target web --out-dir ../wasm ./target/wasm32-unknown-unknown/${TARGET}/game.wasm
