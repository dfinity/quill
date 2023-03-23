p=$(mktemp) 
err=$(mktemp)
"$QUILL" generate --phrase "tornado allow zero warm have deer wool finish tiger ski dynamic strong" --seed-file /dev/null --overwrite-seed-file --pem-file "$p" --overwrite-pem-file &> "$err" || { cat "$err" >&2 && exit 1 ; }
"$QUILL" public-ids --pem-file "$p"