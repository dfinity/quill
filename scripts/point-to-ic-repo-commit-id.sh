#!/bin/bash
set -euo pipefail

# Phase 0: Get ready.

# Require exactly one argument: the new commit ID.
if [[ "$#" -ne 1 ]]; then
    echo "🙅 Oops! You need to provide exactly one argument to this command." >&2
    echo "Usage: ${0} NEW_COMMIT_ID" >&2
    echo "A good place to look for a suitable commit ID is at" >&2
    echo "https://github.com/dfinity/ic/actions/workflows/ci-main.yml?query=branch%3Amaster" >&2
    exit 1
fi
NEW_COMMIT_ID="$1"

# Do not allow unstaged changes. Otherwise, it is harder for the user to see the
# changes they've made vs. the changes made by this script. Ideally, we'd
# support --allow-unstaged-changes flag, but this has not been implemented (yet).
if ! git diff --quiet; then
    echo "🙅 There are unstaged changes." >&2
    echo "Therefore, no files have been modified. Please, stage all changes" >&2
    echo "before running this script again." >&2
    exit 1
fi

# cd to the root of the repo
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Infer the current ic repo commit ID.
ORIGINAL_COMMIT_ID=$(
    grep '{ git = "https://github.com/dfinity/ic", rev = ".*" }' \
         Cargo.toml \
        | head -n 1 \
        | sed 's/.*rev *= *"\(.*\)".*/\1/'
)
echo "Original ic repo commit ID: ${ORIGINAL_COMMIT_ID}" >&2

if [[ "${NEW_COMMIT_ID}" == "${ORIGINAL_COMMIT_ID}" ]]; then
    echo "" >&2
    echo "⚠️ Warning: Looks like there will be no changes here, because" >&2
    echo "the new commit ID appears to be the same as the old one." >&2
    echo "Proceeding anyways." >&2
fi

# Phase 1.1: Update Cargo.toml

echo >&2
sed -i '' \
    -e "s~{ git = \"https://github.com/dfinity/ic\", rev = \".*\"~{ git = \"https://github.com/dfinity/ic\", rev = \"${NEW_COMMIT_ID}\"~g" \
    Cargo.toml
echo "Updated Cargo.toml." >&2
echo >&2


# Phase 1.2: Download .did files.
# From the ic repo to the candid directory as of $NEW_COMMIT_ID.
# (This is more interesting/work step.)

function download_from_ic_repo {
    local source_path="${1}"
    local destination_path="${2}"

    echo "  $(basename ${destination_path})" >&2

    curl \
        --fail \
        --location \
        --silent \
        --output "${destination_path}" \
        "https://raw.githubusercontent.com/dfinity/ic/${NEW_COMMIT_ID}/${source_path}"
}

echo "Updating candid files..." >&2
download_from_ic_repo rs/bitcoin/ckbtc/minter/ckbtc_minter.did   candid/ckbtc_minter.did
download_from_ic_repo rs/nns/governance/canister/governance.did  candid/governance.did
download_from_ic_repo rs/nns/gtc/canister/gtc.did                candid/gtc.did
download_from_ic_repo rs/rosetta-api/icrc1/ledger/ledger.did     candid/icrc1.did
download_from_ic_repo rs/rosetta-api/icp_ledger/ledger.did       candid/ledger.did
download_from_ic_repo rs/registry/canister/canister/registry.did candid/registry.did
download_from_ic_repo rs/sns/governance/canister/governance.did  candid/sns-governance.did
download_from_ic_repo rs/sns/root/canister/root.did              candid/sns-root.did
download_from_ic_repo rs/sns/swap/canister/swap.did              candid/sns-swap.did
download_from_ic_repo rs/nns/sns-wasm/canister/sns-wasm.did      candid/snsw.did
echo "Done updating candid files." >&2
echo >&2

# Phase 1.3: Update end to end test(s).
sed -i '' \
    "s/IC_COMMIT=.*/IC_COMMIT=\"${NEW_COMMIT_ID}\"/" \
    e2e/utils/setup_nns.bash
echo "Done updating end to end tests." >&2
echo >&2

# Phase 1.4: Update Cargo.lock.
echo "Updating Cargo.lock..."
cargo build --quiet
echo "Done." >&2
echo >&2

# Phase 2: Verify tests still pass. Often, this will surface all (or at least
# most of) the Rust code that needs to be updated as a result of pulling in a
# more recent ic commit.
if ! cargo test; then
    # Report partial success. We made updates, but now manual changes are needed.
    echo >&2
    echo "⚠️ Warning: test(s) no longer pass after pointing to ic repo commit" >&2
    echo "${NEW_COMMIT_ID}. Most likely, this is due to updated canister interfaces" >&2
    echo "(in particular new fields in Candid records and variants)." >&2
    echo "You will have to make manual (Rust) code changes in order to get things" >&2
    echo "working again. Alternatively, you can run `git restore .` at the root" >&2
    echo "of this repo to undo the changes made here." >&2
    exit 1
fi

# Finally, report results.
echo >&2
git diff --stat Cargo.toml candid e2e >&2
echo >&2
echo "🎉 Success!" >&2
echo "I have changed Cargo.toml, updated files in the candid dir and updated end to" >&2
echo "end tests. We are now referring to ic repo commit ${NEW_COMMIT_ID}." >&2
echo "These changes have NOT been staged. Therefore, you can inspect them by" >&2
echo "running `git diff`. Once you are satisfied, proceed with the rest of your" >&2
echo "usual git workflow." >&2
