{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  auth = inputs.flake-utils.lib.mkApp {
    drv = nixpkgs.writeShellScriptBin "auth" ''
      export TEMPLATES_DIR=${cell.packages.auth}/share/templates
      export PUBLIC_DIR=${cell.packages.auth}/share/public
      ${cell.packages.auth}/bin/auth
    '';
  };
}
