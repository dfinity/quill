{ rev    ? "7a94fcdda304d143f9a40006c033d7e190311b54"
, sha256 ? "0d643wp3l77hv2pmg2fi7vyxn4rwy0iyr8djcw1h5x72315ck9ik"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "quill";
  version = "0.2.17";

  src = ./.;

  cargoSha256 = "sha256-J66D1j1AdQanD9FuI+OuXalVbX3sYzIpSSRdIt10/TE=";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rls rustfmt clippy pkg-config ];
  buildInputs = [ openssl protobuf ]
    ++ (lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  ic = fetchFromGitHub {
    owner = "dfinity";
    repo = "ic";
    rev = "c7c002be1f49482f920d22b3ec561331edacc6f8";
    sha256 = "1v3w0r0kfwzhmwkyar0rha8s98nfc9cjpi90gx9f02xdx0wz8hxm";
    # date = "2022-08-17T09:06:20+00:00";
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
