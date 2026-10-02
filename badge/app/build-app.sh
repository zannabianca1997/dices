#!/usr/bin/env bash
# Build the Tactility app (.app package) with the prebuilt Tactility SDK.
#
# Usage:
#   ./build-app.sh            build for esp32s3
#   ./build-app.sh <ip>       build for esp32s3 and install to the badge
#                             (requires the badge's Development service)
#
# Runs inside the espressif/idf:v6.1 container (the Tactility 0.8.0-dev SDK
# is built against it). The Rust staticlib must exist first: run
# ./build-rust.sh xtensa-esp32s3-espidf (see Dockerfile.rust).
set -euo pipefail

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
SDK_VERSION="0.8.0-dev"

# Fetch the build tool once (it is developed inside the Tactility repo)
TOOL="${APP_DIR}/tools/tactility.py"
if [[ ! -f "${TOOL}" ]]; then
    echo ">> fetching tactility.py"
    mkdir -p "${APP_DIR}/tools"
    curl -sL "https://raw.githubusercontent.com/TactilityProject/Tactility/main/tactility.py" \
        -o "${TOOL}"
fi

# esp32s3 only: the badge target (matches manifest.properties platforms)
TARGET="esp32s3"

echo ">> building app for ${TARGET}"
# Mount the repo root at /work: generated/link.cmake refers to the staticlib
# via /work-target paths (build-rust.sh runs with the same layout)
REPO_ROOT="$(cd "${APP_DIR}/../.." && pwd)"
docker run --rm \
    -v "${REPO_ROOT}:/work" \
    -w /work/badge/app \
    dices-rust-idf:6.1 \
    bash -c "python tools/tactility.py build --architecture ${TARGET}"

PACKAGE=$(ls "${APP_DIR}/build/"*.app 2>/dev/null | head -n1 || true)
if [[ -z "${PACKAGE}" ]]; then
    echo "error: no .app produced" >&2
    exit 1
fi
echo ">> built: ${PACKAGE}"

if [[ $# -ge 1 ]]; then
    echo ">> installing to $1"
    docker run --rm \
        -v "${REPO_ROOT}:/work" \
        -w /work/badge/app \
        --network host \
        dices-rust-idf:6.1 \
        python tools/tactility.py install --host "$1"
fi
