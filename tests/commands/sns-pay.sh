"$QUILL" sns pay --amount-icp-e8s 100000 --subaccount e000d80101 --ticket-creation-time 1675815621 --ticket-id 100 --canister-ids-file ./sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run -
