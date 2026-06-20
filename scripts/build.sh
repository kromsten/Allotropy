
#!/bin/bash
set -e

# Determine the script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

cd "$PROJECT_ROOT"

cargo build --release --target wasm32-unknown-unknown --workspace

mkdir -p artifacts


ARCH=$( [[ $(uname -m) == "arm64" ]] && echo "aarch64" || echo "x86_64" )

for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
    name=$(basename "$wasm" .wasm)
    wasm-opt -Oz \
      --enable-bulk-memory \
      --enable-mutable-globals \
      --enable-sign-ext \
      --enable-nontrapping-float-to-int \
      --enable-reference-types \
      "$wasm" -o "artifacts/${name}-${ARCH}.wasm"
done

echo "All contracts optimized into artifacts/"