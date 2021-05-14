{ pkgs ? import ./nix {} }:
let
  packages = import ./. { inherit pkgs; };
in
pkgs.mkCompositeShell {
  name = "gltk-env";
  inputsFrom = pkgs.stdenv.lib.attrValues packages.shells;
  nativeBuildInputs = [ pkgs.nix-fmt ];
}
