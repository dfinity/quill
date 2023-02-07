PRINCIPAL=fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae
"$QUILL" sns transfer $PRINCIPAL --amount 0.000123 --canister-ids-file ./sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run -
