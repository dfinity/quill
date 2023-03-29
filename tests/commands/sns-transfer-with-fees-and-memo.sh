PRINCIPAL=fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae
"$QUILL" sns transfer $PRINCIPAL --amount 123.0456 --fee 0.0023 --memo 777 --canister-ids-file ./sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run -
