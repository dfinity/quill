PEM=`cat ./identity.pem`

tests=0
for f in `ls -1 ./commands/| sort -n`; do
    expected=`cat outputs/${f/sh/txt}`
    out=`echo "$PEM" | sh "commands/$f"`
    if [ "$out" != "$expected" ]; then
        >&2 echo "Test case $f failed." 
        >&2 echo "Expected output:"
        >&2 echo "$expected"
        >&2 echo "Generated output:"
        >&2 echo "$out"
        exit 1
    fi
    tests=$((tests + 1))
done

echo "✅ All $tests tests succeeded!"
