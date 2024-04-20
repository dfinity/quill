load ../utils/_

setup() {
    standard_nns_setup
}

teardown() {
    standard_nns_teardown
}

@test "basic create neuron" {
    #account is initialized with 10_000 tokens
    assert_command quill account-balance 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 --yes --insecure-local-dev-mode
    assert_string_match 'Balance: 1000000000.00000000 ICP'

    # stake 3 tokens
    assert_command bash -c "quill neuron-stake --amount 3 --name myneur --pem-file \"$PEM_LOCATION/identity.pem\" > stake.call"
    assert_file_not_empty stake.call
    SEND_OUTPUT="$(quill send stake.call --yes --insecure-local-dev-mode)"
    assert_command echo "$SEND_OUTPUT" # replay the output so string matches work
    echo "$SEND_OUTPUT"
    assert_string_match "Method name: claim_or_refresh_neuron_from_account"
    NEURON_ID="$(echo "$SEND_OUTPUT" | grep -E ' neuron ' | sed 's/[^0-9]//g')"
    echo "NEURON: $NEURON_ID"
    assert_string_match "
Successfully staked ICP in neuron " #fragment of a correct response

    # check that staking worked using get-neuron-info
    assert_command bash -c "quill get-neuron-info \"$NEURON_ID\" --yes --insecure-local-dev-mode"
    assert_string_match 'Total stake: 3.00000000 ICP'

    # increase dissolve delay by 6 months
    assert_command bash -c "quill neuron-manage --additional-dissolve-delay-seconds 15778800 --pem-file \"$PEM_LOCATION/identity.pem\" \"$NEURON_ID\" > more-delay.call"
    assert_file_not_empty more-delay.call
    assert_command quill send more-delay.call --yes --insecure-local-dev-mode
    assert_string_match 'Neuron successfully configured'

    # check that increasing dissolve delay worked, this time using list-neurons
    assert_command bash -c "quill list-neurons --pem-file \"$PEM_LOCATION/identity.pem\" > neuron.call"
    assert_command quill send neuron.call --yes --insecure-local-dev-mode
    assert_string_match "Dissolve delay: 6 months"
}
