LEDGER_ACCOUNT_ID=ec0e2456fb9ff6c80f1d475b301d9b2ab873612f96e7fd74e7c0c0b2d58e6693
${CARGO_TARGET_DIR:-../target}/debug/sns-quill --canister-ids-file ./canister_ids.json  account-balance $LEDGER_ACCOUNT_ID --dry-run
