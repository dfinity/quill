{ rev    ? "8e1eab9eae4278c9bb1dcae426848a581943db5a"
, sha256 ? "0jf4rccc0z9in1iahw13i5wl93gbp1x6mkjd3qivjg97ms9qw3l0"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "quill";
  version = "0.2.14";

  src = ./.;

  cargoSha256 = "sha256-t+N0UoVOJIxi1q0SBfCqxNIcmXVBo44hhKE0SllJ63M=";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rls rustfmt clippy pkg-config ];
  buildInputs = [ openssl protobuf ]
    ++ (lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  ic = fetchFromGitHub {
    owner = "dfinity";
    repo = "ic";
    rev = "779549eccfcf61ac702dfc2ee6d76ffdc2db1f7f";
    sha256 = "1r31d5hab7k1n60a7y8fw79fjgfq04cgj9krwa6r9z4isi3919v6";
  };

  registry = "file://local-registry";

  preBuild = ''
    export REGISTRY_TRANSPORT_PROTO_INCLUDES=${ic}/rs/registry/transport/proto
    export IC_BASE_TYPES_PROTO_INCLUDES=${ic}/rs/types/base_types/proto
    export IC_PROTOBUF_PROTO_INCLUDES=${ic}/rs/protobuf/def
    export IC_NNS_COMMON_PROTO_INCLUDES=${ic}/rs/nns/common/proto

    export PROTOC=${protobuf}/bin/protoc
    export OPENSSL_DIR=${openssl.dev}
    export OPENSSL_LIB_DIR=${openssl.out}/lib
  '';

  meta = with lib; {
    description = "Minimalistic ledger and governance toolkit for cold wallets.";
    homepage = https://github.com/dfinity/quill;
    license = licenses.asl20;
    maintainers = [ maintainers.jwiegley ];
    platforms = platforms.all;
  };
}
