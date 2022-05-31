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
    assert_match 'Response: \(record { e8s = 1_000_000_000_000 : nat64 }\)'

    # stake 3
    assert_command bash -c "quill --pem-file $PEM_LOCATION/identity.pem neuron-stake --amount 3 --name myneur > call.temp"
    assert_file_not_empty call.temp
    assert_command quill send call.temp --yes
    assert_match "Method name: send_dfx"
    assert_match "Method name: claim_or_refresh_neuron_from_account"
    assert_match "NeuronId = record { id =" #fragment of the correct response

    # balance reduced
}
