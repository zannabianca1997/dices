#!/usr/bin/env bash
# Build the Rust core (dices-badge staticlib) for a target and generate
# generated/link.cmake for the Tactility app to link against.
#
# Usage: ./build-rust.sh [target-triple]
#   (default: the host target, for quick smoke builds)
#
# ESP-IDF targets (e.g. xtensa-esp32s3-espidf) are built with
# `-Z build-std` (the espup toolchain ships rust-src instead of precompiled
# Xtensa std); run this where the espup toolchain is available
# (see Dockerfile.rust).
set -euo pipefail

TARGET="${1:-}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP_DIR="${REPO_ROOT}/badge/app"
GENERATED_DIR="${APP_DIR}/generated"
mkdir -p "${GENERATED_DIR}"

TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target}"

PROFILE="badge"
CARGO_ARGS=(--manifest-path "${REPO_ROOT}/Cargo.toml" -p dices-badge --profile "${PROFILE}")
BUILD_STD=0
if [[ -n "${TARGET}" ]]; then
    CARGO_ARGS+=(--target "${TARGET}")
    case "${TARGET}" in
        *espidf*)
            # No precompiled std for ESP-IDF targets: build it from source.
            # The target only supports the abort panic strategy; the engine's
            # abort feature is disabled there (see dices-engine's Context::abort
            # and dices-std's std.repl.abort) and compiles fine under abort.
            BUILD_STD=1
            CARGO_ARGS+=("-Z" "build-std=std,panic_abort")
            # Strip panic formatting from std: big savings, and dices errors
            # are values, not panics, so nothing user-facing is lost
            ;;
    esac
fi

echo ">> cargo build ${CARGO_ARGS[*]}"
cargo build "${CARGO_ARGS[@]}"

# Locate the built staticlib and rlib
if [[ -n "${TARGET}" ]]; then
    ARTIFACT_DIR="${TARGET_DIR}/${TARGET}/${PROFILE}"
else
    ARTIFACT_DIR="${TARGET_DIR}/${PROFILE}"
fi
STATICLIB="${ARTIFACT_DIR}/libdices_badge.a"
# The Tactility app link is `-shared` (the OS loader requires ELF_TYPE_DYN and
# relocates the image in RAM at load time). Optimized Rust emits address
# constants (vtable entries, drop glue) in read-only `.literal`/`.rodata`
# sections, and the Xtensa GNU ld refuses to place the resulting dynamic
# relocations there ("dangerous relocation"). Mark those input sections as
# writable data so ld emits R_XTENSA_RELATIVE into .rela.dyn; esp_elf applies
# them after copying the segments into RAM.
OBJCOPY="$(dirname "$(which xtensa-esp32s3-elf-gcc || which gcc)")/xtensa-esp32s3-elf-objcopy"
if [[ ! -x "${OBJCOPY}" ]]; then
    OBJCOPY="$(command -v xtensa-esp32s3-elf-objcopy || command -v objcopy)"
fi
echo ">> marking .literal/.rodata writable in the staticlib (${OBJCOPY})"
"${OBJCOPY}" --wildcard \
    --set-section-flags '.literal*=alloc,load,data' \
    --set-section-flags '.rodata*=alloc,load,data' \
    --set-section-flags 'linkme_*=alloc,load,data' \
    --set-section-flags 'linkm2_*=alloc,load,data' \
    "${STATICLIB}"

echo ">> staticlib: ${STATICLIB}"

# Rust std archives the consumer must link after the staticlib.
#
# rustc used to report these via a `native-static-libs` note, which current
# rustc no longer prints for staticlib builds, so instead we collect the
# archives ourselves (the linker skips unused members) and link them inside a
# linker group, which makes the order irrelevant.
#
# With -Z build-std (ESP-IDF targets) std lives in the build's deps directory,
# next to the crate dependencies; without it, in the toolchain's rustlib dir.
# Only the std closure is collected: feeding every crate rlib to the linker
# makes --gc-sections scan hundreds of megabytes for nothing.
if [[ "${BUILD_STD}" == 1 ]]; then
    LIB_DIR="${ARTIFACT_DIR}/deps"
    declare -A NEWEST
    STD_PATTERNS=(
        "libstd-*.rlib" "libcore-*.rlib" "liballoc-*.rlib"
        "libpanic_abort-*.rlib" "libpanic_unwind-*.rlib" "libcompiler_builtins-*.rlib"
        "librustc_std_workspace_*.rlib" "libstd_detect-*.rlib"
        "libhashbrown-*.rlib" "liblibc-*.rlib" "libcfg_if-*.rlib"
        "libunwind-*.rlib" "libmemchr-*.rlib" "libminiz_oxide-*.rlib"
        "libadler2-*.rlib" "libaddr2line-*.rlib" "libgimli-*.rlib"
        "libobject-*.rlib" "librustc_demangle-*.rlib"
    )
    ARCHIVES=()
    for pattern in "${STD_PATTERNS[@]}"; do
        for archive in "${LIB_DIR}"/${pattern}; do
            [[ -e "${archive}" ]] || continue
            # Interrupted builds with different flags leave several hash
            # variants of the same crate behind; keep only the newest of each
            # so the linker does not grind through hundreds of megabytes.
            name="$(basename "${archive}" .rlib)"
            name="${name%-*}"
            newest="${NEWEST[$name]:-}"
            if [[ -z "${newest}" || "${archive}" -nt "${newest}" ]]; then
                NEWEST[$name]="${archive}"
            fi
        done
    done
    ARCHIVES=("${NEWEST[@]}")
else
    SYSROOT="$(rustc ${TARGET:+--target "${TARGET}"} --print sysroot)"
    if [[ -n "${TARGET}" ]]; then
        LIB_DIR="${SYSROOT}/lib/rustlib/${TARGET}/lib"
    else
        HOST="$(rustc -vV | grep '^host: ' | cut -d' ' -f2)"
        LIB_DIR="${SYSROOT}/lib/rustlib/${HOST}/lib"
    fi
    ARCHIVES=("${LIB_DIR}"/*.rlib "${LIB_DIR}"/*.a)
fi
if [[ ! -d "${LIB_DIR}" ]]; then
    echo "error: Rust std directory not found: ${LIB_DIR}" >&2
    exit 1
fi
echo ">> std libs from: ${LIB_DIR} (${#ARCHIVES[@]} archives)"

CMAKE_LIBS="\"-Wl,--start-group\""
for archive in "${ARCHIVES[@]}"; do
    CMAKE_LIBS+=" \"${archive}\""
done
CMAKE_LIBS+=" \"-Wl,--end-group\""

cat > "${GENERATED_DIR}/link.cmake" <<EOF
# Generated by build-rust.sh - do not edit
set(DICES_STATICLIB "${STATICLIB}")
set(DICES_LINK_LIBS ${CMAKE_LIBS})
EOF

echo ">> wrote ${GENERATED_DIR}/link.cmake ($(wc -w <<< "${CMAKE_LIBS}") entries)"
