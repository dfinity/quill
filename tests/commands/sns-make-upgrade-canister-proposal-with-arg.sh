PROPOSER_NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069

#$ didc encode '(record {major=2:nat32; minor=3:nat32;})' --format blob
#blob "DIDL\01l\02\b9\fa\ee\18y\b5\f6\a1Cy\01\00\02\00\00\00\03\00\00\00"

"$QUILL" sns \
        make-upgrade-canister-proposal \
        --wasm-path outputs/sns_canister.wasm \
        --canister-upgrade-arg "(record {major=2:nat32; minor=3:nat32;})" \
        --target-canister-id pycv5-3jbbb-ccccc-ddddd-cai \
        $PROPOSER_NEURON_ID \
        --canister-ids-file ./sns_canister_ids.json \
        --pem-file - \
    | "$QUILL" \
        send \
        --dry-run \
        -
