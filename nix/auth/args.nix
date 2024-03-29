{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  craneLib = inputs.crane.lib;
in
rec {
  crane = craneLib.overrideToolchain inputs.cells.rust.toolchain.rust;

  crateName = crane.crateNameFromCargoToml { cargoToml = "${src}/Cargo.toml"; };

  commonArgs = {
    inherit src;
    inherit (crateName) pname version;
  };

  src = std.incl (inputs.self + /auth) [
    (inputs.self + /auth/Cargo.toml)
    (inputs.self + /auth/Cargo.lock)
    (inputs.self + /auth/src)
  ];
}
