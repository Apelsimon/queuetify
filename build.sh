#!/bin/bash

ROOT="$(git rev-parse --show-toplevel)"
OUTPUT_DIR="${ROOT}/build"
PUBLIC_DIR="${OUTPUT_DIR}/public"

rm -rf ${OUTPUT_DIR}

mkdir -p "${PUBLIC_DIR}"

cd client
npm run build
mv dist/* "${PUBLIC_DIR}"

cd ${ROOT}/server
cargo build
cp target/debug/server ${OUTPUT_DIR}
cp .env.local ${OUTPUT_DIR}