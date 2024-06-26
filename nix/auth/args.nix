{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  craneLib = inputs.crane.lib;
in
rec {
  crane = craneLib.overrideToolchain inputs.cells.rust.toolchain.rust;

  crateName =
    crane.crateNameFromCargoToml { cargoToml = "${src}/auth/Cargo.toml"; };

  commonWorkspaceArgs = {
    inherit src;
    inherit (crateName) pname version;
  };

  commonArgs = commonWorkspaceArgs // { cargoExtraArgs = "-p auth"; };

  src =
    let
      src = std.incl (inputs.self) [
        (inputs.self + /auth/Cargo.toml)
        (inputs.self + /auth/src)
        (inputs.self + /Cargo.toml)
        (inputs.self + /Cargo.lock)
        (inputs.self + /featurize/Cargo.toml)
      ];
      dummyMain = "pub fn main() {}";
    in
    nixpkgs.stdenv.mkDerivation {
      name = "src";
      src = src;
      installPhase = ''
        mkdir -p $out/featurize/src/
        cp -r * $out/
        echo "${dummyMain}" > $out/featurize/src/main.rs
      '';
    };
}
