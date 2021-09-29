{ rev    ? "a3a23d9599b0a82e333ad91db2cdc479313ce154"
, sha256 ? "05xmgrrnw6j39lh3d48kg064z510i0w5vvrm1s5cdwhdc2fkspjq"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "quill";
  version = "0.2.8";

  src = ./.;

  cargoSha256 = "1vqjkzdyikk9x26anzha3pkpgj7p7ms72yv664wb6711gar9yj15";

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
