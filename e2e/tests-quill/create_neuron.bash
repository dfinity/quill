load ../utils/_

setup() {
    standard_nns_setup
}

teardown() {
    standard_nns_teardown
}

@test "basic create neuron" {
    #account is initialized with 10_000 tokens
    assert_command quill account-balance 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752
    assert_match "Sending message with Call type: query"
    assert_string_match 'Response: (record { e8s = 1_000_000_000_000 : nat64 })'

    # stake 3 tokens
    assert_command bash -c "quill --pem-file $PEM_LOCATION/identity.pem neuron-stake --amount 3 --name myneur > stake.call"
    assert_file_not_empty stake.call
    assert_command quill send stake.call --yes
    assert_match "Method name: send_dfx"
    assert_match "Method name: claim_or_refresh_neuron_from_account"
    assert_string_match "record { result = opt variant { NeuronId = record { id =" #fragment of a correct response

    # balance reduced
    assert_command bash -c "quill --pem-file $PEM_LOCATION/identity.pem list-neurons > neuron.call"
    assert_file_not_empty neuron.call
    assert_command quill send neuron.call --yes
    assert_string_match 'stake_e8s = 300_000_000'
}
