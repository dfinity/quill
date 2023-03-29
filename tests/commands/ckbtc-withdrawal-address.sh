err=$(mktemp)
"$QUILL" ckbtc withdrawal-address --pem-file - 2> "$err" || { cat "$err" >&2 && exit 1; }