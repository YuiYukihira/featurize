{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  cargoArtifacts = cell.packages.featurize-deps;
in
with cell.args; rec {
  featurize = cell.packages.featurize;
  featurize-clippy =
    crane.cargoClippy (commonArgs // { inherit cargoArtifacts; });
  featurize-fmt = crane.cargoFmt commonWorkspaceArgs;
  featurize-audit = crane.cargoAudit {
    inherit src;
    advisory-db = inputs.advisory-db;
  };
  featurize-nextest = crane.cargoNextest (commonWorkspaceArgs // {
    inherit cargoArtifacts;
    partitions = 1;
    partitionType = "count";
    cargoNextestExtraArgs = "-p featurize";
  });
}
