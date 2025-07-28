#! /bin/bash

set -euo pipefail

IC_COMMIT="e915efecc8af90993ccfc499721ebe826aadba60"

if [[ -z "${DOWNLOAD_DIR:-}" ]]; then
  DOWNLOAD_DIR=$(mktemp -d -t dfx-e2e-XXXXXXXX)
else
  echo "DOWNLOAD DIR is ${DOWNLOAD_DIR}."
fi

get_binary() {
  local FILENAME
  FILENAME="$1"
  if test -e "$DOWNLOAD_DIR/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  local OS
  OS="$(uname)"
  case "$OS" in
  Darwin)
    echo "fetching from: https://download.dfinity.systems/ic/${IC_COMMIT}/binaries/x86_64-darwin/${FILENAME}.gz"
    curl "https://download.dfinity.systems/ic/${IC_COMMIT}/binaries/x86_64-darwin/${FILENAME}.gz" | gunzip >"$TMP_FILE"
    ;;
  Linux)
    echo "fetching from: https://download.dfinity.systems/ic/${IC_COMMIT}/binaries/x86_64-linux/${FILENAME}.gz"
    curl "https://download.dfinity.systems/ic/${IC_COMMIT}/binaries/x86_64-linux/${FILENAME}.gz" | gunzip >"$TMP_FILE"
    ;;
  *)
    printf "ERROR: %s '%s'\n" \
      "Cannot download binary" "$FILENAME" \
      "Unsupported platform:" "$OS" \
      >&2
    exit 1
    ;;
  esac
  install -m 755 "$TMP_FILE" "$DOWNLOAD_DIR/$FILENAME"
  rm "$TMP_FILE"
}

get_wasm() {
  local FILENAME
  FILENAME="$1"
  if test -e "$DOWNLOAD_DIR/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  echo "fetching from: https://download.dfinity.systems/ic/${IC_COMMIT}/canisters/${FILENAME}.gz"
  curl "https://download.dfinity.systems/ic/${IC_COMMIT}/canisters/${FILENAME}.gz" | gunzip >"$TMP_FILE"
  install -m 644 "$TMP_FILE" "$DOWNLOAD_DIR/$FILENAME"
  rm "$TMP_FILE"
}

get_binary ic-nns-init
get_wasm registry-canister.wasm
get_wasm governance-canister.wasm
get_wasm governance-canister_test.wasm
get_wasm ledger-canister_notify-method.wasm
get_wasm root-canister.wasm
get_wasm cycles-minting-canister.wasm
get_wasm lifeline_canister.wasm
get_wasm genesis-token-canister.wasm
get_wasm identity-canister.wasm
get_wasm nns-ui-canister.wasm
get_wasm sns-wasm-canister.wasm
get_wasm ic-icrc1-ledger.wasm
get_wasm ic-ckbtc-minter.wasm

NNS_URL="http://localhost:$(cat "$E2E_NETWORK_DATA_DIRECTORY/pocket-ic-port")/instances/0/"

"${DOWNLOAD_DIR}/ic-nns-init" \
  --url "$NNS_URL" \
  --pass-specified-id \
  --initialize-ledger-with-test-accounts 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 76374de112443a5415f4bef978091a622b8f41035c99147abc1471fd99635661 \
  --wasm-dir "$DOWNLOAD_DIR"
  