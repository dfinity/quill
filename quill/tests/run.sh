#!/usr/bin/env bash
PEM=`cat ./identity.pem`

set -euo pipefail

tests=0
for f in `ls -1 ./commands/| sort -n`; do
    expected="outputs/${f/sh/txt}"
    out=$(mktemp)
    echo "$PEM" | sh "commands/$f" > "$out"
    if ! diff -r --ignore-all-space "$expected" "$out" >/dev/null; then
        >&2 echo "Test case $f failed." 
        >&2 echo "Expected output:"
        >&2 cat "$expected"
        >&2 echo
        >&2 echo "Generated output:"
        >&2 cat "$out"
        >&2 echo
        exit 1
    fi
    tests=$((tests + 1))
done

echo "✅ All $tests tests succeeded!"
