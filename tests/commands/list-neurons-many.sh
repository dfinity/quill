${CARGO_TARGET_DIR:-../target}/debug/quill list-neurons 123 456 789 --pem-file - | ${CARGO_TARGET_DIR:-../target}/debug/quill send --dry-run -
