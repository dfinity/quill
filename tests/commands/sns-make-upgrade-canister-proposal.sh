PROPOSER_NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069

"$QUILL" sns \
    make-upgrade-canister-proposal \
    --wasm-path=outputs/sns_canister.wasm \
    --target-canister-id=pycv5-3jbbb-ccccc-ddddd-cai \
    --canister-ids-file=./sns_canister_ids.json \
    --pem-file=- \
    $PROPOSER_NEURON_ID \
    | "$QUILL" send --dry-run -

