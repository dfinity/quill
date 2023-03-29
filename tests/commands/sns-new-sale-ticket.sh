"$QUILL" sns new-sale-ticket --amount-icp-e8s 100000 --subaccount e000d80101 --canister-ids-file ./sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run -
