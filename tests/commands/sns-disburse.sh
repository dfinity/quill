NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
"$QUILL" sns disburse $NEURON_ID --to fdsgv-62ihb-nbiqv-xgic5-iefsv-3cscz-tmbzv-63qd5-vh43v-dqfrt-pae --amount 50 --canister-ids-file sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run
