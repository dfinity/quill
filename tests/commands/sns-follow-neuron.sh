NEURON_ID=83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
FOLLOWEE=75c606cac8d0dab0a6f5db99d64c9b5312ed8cca2f971ea0ea960926db530d7f
"$QUILL" sns follow-neuron $NEURON_ID --type transfer-sns-treasury-funds --followees $FOLLOWEE --canister-ids-file sns_canister_ids.json --pem-file - | "$QUILL" send --dry-run
