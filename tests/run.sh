#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

PEM=$(cat ../e2e/assets/identity.pem)

run_test() {
    cmd=$(basename "$1")
    expected="outputs/${cmd/sh/txt}"
    out=$(mktemp)
    export QUILL="${CARGO_TARGET_DIR:-../target}/debug/quill"
    echo "$PEM" | sh -o pipefail "commands/$cmd" > "$out"
    if ! diff -r --ignore-all-space "$expected" "$out" >/dev/null; then
        >&2 echo "Test case $cmd failed." 
        >&2 echo "Expected output:"
        >&2 cat "$expected"
        >&2 echo
        >&2 echo "Generated output:"
        >&2 cat "$out"
        >&2 echo
        exit 1
    fi
    tests=$((tests + 1))
}

tests=0
if [ "$*" ]; then
    for f; do
        run_test "$f"
    done
else
    for f in ./commands/*; do
        run_test "$f"
    done
fi

echo "✅ All $tests tests succeeded!"
