#!/bin/bash

ROOT="$(git rev-parse --show-toplevel)"
OUTPUT_DIR="${ROOT}/build"
TEMPLATE_DIR="${OUTPUT_DIR}/templates"

rm -rf ${OUTPUT_DIR}

mkdir -p "${OUTPUT_DIR}"
mkdir -p "${TEMPLATE_DIR}"

# ---- Client ----
cd client
npm run build
cp dist/*.js "${OUTPUT_DIR}"

# ---- Server ----
cd ${ROOT}/server
cargo build
cp target/debug/server ${OUTPUT_DIR}
cp -r templates ${OUTPUT_DIR}
cp -r configuration ${OUTPUT_DIR}
cp .env* ${OUTPUT_DIR}