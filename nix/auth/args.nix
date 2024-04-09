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

  commonArgs = {
    inherit src;
    inherit (crateName) pname version;
  };

  src = std.incl (inputs.self) [
    (inputs.self + /auth/Cargo.toml)
    (inputs.self + /auth/src)
    (inputs.self + /Cargo.toml)
    (inputs.self + /Cargo.lock)
  ];
}
