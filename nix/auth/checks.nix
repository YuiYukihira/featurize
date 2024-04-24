{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  cargoArtifacts = cell.packages.auth-deps;
in
with cell.args; rec {
  auth = cell.packages.auth;
  auth-clippy = crane.cargoClippy (commonArgs // { inherit cargoArtifacts; });
  auth-fmt = crane.cargoFmt commonWorkspaceArgs;
  auth-audit = crane.cargoAudit {
    inherit src;
    advisory-db = inputs.advisory-db;
  };
  auth-nextest = crane.cargoNextest (commonWorkspaceArgs // {
    inherit cargoArtifacts;
    partitions = 1;
    partitionType = "count";
    cargoNextestExtraArgs = "-p auth";
  });
}
