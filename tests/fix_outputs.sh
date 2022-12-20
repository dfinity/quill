#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

PEM=$(cat ../e2e/assets/identity.pem)

fixup() {
    cmd=$(basename "$1")
    out=${cmd/sh/txt}
    export QUILL="${CARGO_TARGET_DIR:-../target}/debug/quill"
    echo "$PEM" | bash -o pipefail "commands/$f" > "./outputs/$out"
}

if [ "$*" ]; then
    for f; do
        fixup "$f"
    done
else
    for f in ./commands/*; do
        fixup "$f"
    done
fi
