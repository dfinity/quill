load "${BATSLIB}"/load.bash
load ../utils/assertions

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT="${BATS_TEST_DIRNAME}/../assets/$1/"
    cp -R "$ASSET_ROOT"/* .

    if [ -f ./patch.bash ]; then
        # shellcheck disable=SC1091
        source ./patch.bash
    fi
}

standard_setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export DFX_E2E_TEMP_DIR="$x"

    if [ "$(uname)" == "Darwin" ]; then
        project_relative_path="Library/Application Support/org.dfinity.dfx"
    elif [ "$(uname)" == "Linux" ]; then
        project_relative_path=".local/share/dfx"
    fi

    mkdir "$x/working-dir"
    mkdir "$x/cache-root"
    mkdir "$x/config-root"
    mkdir "$x/home-dir"

    # we need to configure dfxvm in the isolated home directory
    default_dfx_version="$(dfxvm default)"
    # don't re-download dfx for every test
    mkdir -p "$x/home-dir/$project_relative_path"
    ln -s "$HOME/$project_relative_path/versions" "$x/home-dir/$project_relative_path/versions"

    cd "$x/working-dir" || exit

    export HOME="$x/home-dir"
    export DFX_CACHE_ROOT="$x/cache-root"
    export DFX_CONFIG_ROOT="$x/config-root"
    export RUST_BACKTRACE=1
    export PEM_LOCATION="${BATS_TEST_DIRNAME}/../assets"
    export E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY="$HOME/$project_relative_path/network/local"
    export E2E_NETWORKS_JSON="$DFX_CONFIG_ROOT/.config/dfx/networks.json"

    dfxvm default "$default_dfx_version"
}

standard_nns_setup() {
    standard_setup
    cp "${BATS_TEST_DIRNAME}/../assets/minimum_dfx.json" dfx.json
    mkdir -p "$(dirname "$E2E_NETWORKS_JSON")"
    cp "${BATS_TEST_DIRNAME}/../assets/minimum_networks.json" "$E2E_NETWORKS_JSON"
    dfx_start "$@"
    NO_CLOBBER="1" "$BATS_TEST_DIRNAME"/../utils/setup_nns.bash
    IC_URL="http://localhost:$(< "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port")"
    export IC_URL
}

standard_nns_teardown() {
    dfx_stop
    rm -rf "$DFX_E2E_TEMP_DIR" || rm -rf "$DFX_E2E_TEMP_DIR"
}

dfx_patchelf() {
    # Don't run this function during github actions
    [ "$GITHUB_ACTIONS" ] && return 0

    # Only run this function on Linux
    (uname -a | grep Linux) || return 0
    echo "dfx = $(which dfx)"

    local CACHE_DIR LD_LINUX_SO
    CACHE_DIR="$(dfx cache show)"
    dfx cache install

    # Both ldd and iconv are providedin glibc.bin package
    LD_LINUX_SO="$(ldd "$(which iconv)"|grep ld-linux-x86|cut -d' ' -f3)"
    for binary in ic-starter icx-proxy replica; do
        local BINARY IS_STATIC USE_LIB64
        BINARY="${CACHE_DIR}/${binary}"
        test -f "$BINARY" || continue
        IS_STATIC="$(ldd "${BINARY}" | grep 'not a dynamic executable')"
        USE_LIB64="$(ldd "${BINARY}" | grep '/lib64/ld-linux-x86-64.so.2')"
        chmod +rw "${BINARY}"
        test -n "$IS_STATIC" || test -z "$USE_LIB64" || patchelf --set-interpreter "${LD_LINUX_SO}" "${BINARY}"
    done
}

determine_network_directory() {
    # not perfect: dfx.json can actually exist in a parent
    if [ -f dfx.json ] && [ "$(jq .networks.local dfx.json)" != "null" ]; then
        echo "found dfx.json with local network in $(pwd)"
        data_dir="$(pwd)/.dfx/network/local"
        wallets_json="$(pwd)/.dfx/local/wallets.json"
        dfx_json="$(pwd)/dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$data_dir"
        export E2E_NETWORK_WALLETS_JSON="$wallets_json"
        export E2E_ROUTE_NETWORKS_JSON="$dfx_json"
    else
        echo "no dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY"
        export E2E_NETWORK_WALLETS_JSON="$E2E_NETWORK_DATA_DIRECTORY/wallets.json"
        export E2E_ROUTE_NETWORKS_JSON="$E2E_NETWORKS_JSON"
    fi
}

# Start the replica in the background.
dfx_start() {
    dfx_patchelf

    if [ "$GITHUB_WORKSPACE" ]; then
        # no need for random ports on github workflow; even using a random port we sometimes
        # get 'address in use', so the hope is to avoid that by using a fixed port.
        FRONTEND_HOST="127.0.0.1:8000"
    else
        # Start on random port for parallel test execution (needed on nix/hydra)
        FRONTEND_HOST="127.0.0.1:0"
    fi
    determine_network_directory
    # Bats creates a FD 3 for test output, but child processes inherit it and Bats will
    # wait for it to close. Because `dfx start` leaves child processes running, we need
    # to close this pipe, otherwise Bats will wait indefinitely.
    if [[ "$*" == "" ]]; then
        dfx start --background --host "$FRONTEND_HOST" 3>&- # Start on random port for parallel test execution
    else
        dfx start --background "$@" 3>&-
    fi

    local dfx_config_root port webserver_port
    dfx_config_root="$E2E_NETWORK_DATA_DIRECTORY/replica-configuration"
    printf "Configuration Root for DFX: %s\n" "${dfx_config_root}"
    test -f "${dfx_config_root}/replica-1.port"
    port=$(cat "${dfx_config_root}/replica-1.port")

    # Overwrite the default networks.local.bind 127.0.0.1:8000 with allocated port
    webserver_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/webserver-port")

    printf "Replica Configured Port: %s\n" "${port}"
    printf "Webserver Configured Port: %s\n" "${webserver_port}"

    if ! timeout 5 sh -c \
        "until nc -z localhost \"${port}\"; do echo \"waiting for replica\"; sleep 1; done"
    then
        echo "could not connect to replica on port ${port}"
        exit 1
    fi
}

wait_until_replica_healthy() {
    echo "waiting for replica to become healthy"
    (
        # dfx ping has side effects, like creating a default identity.
        DFX_CONFIG_ROOT="$DFX_E2E_TEMP_DIR/dfx-ping-tmp"
        dfx ping --wait-healthy
    )
    echo "replica became healthy"
}

# Stop the replica and verify it is very very stopped.
dfx_stop() {
    # to help tell if other icx-proxy processes are from this test:
    echo "pwd: $(pwd)"
    # A suspicion: "address already is use" errors are due to an extra icx-proxy process.
    echo "icx-proxy processes:"
    pgrep icx-proxy || echo "no pgrep/icx-proxy output"

    dfx stop
    local dfx_root=.dfx/
    rm -rf $dfx_root

    # Verify that processes are killed.
    assert_no_dfx_start_or_replica_processes
}
