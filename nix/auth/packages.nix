{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  cargoArtifacts = cell.packages.auth-deps;
in
with cell.args; rec {
  auth-deps = crane.buildDepsOnly commonArgs;

  auth =
    let
      templates = std.incl (inputs.self + /auth) [
        (inputs.self + /auth/templates)
        (inputs.self + /auth/public)
        (inputs.self + /auth/src/input.css)
      ];
      data = nixpkgs.stdenv.mkDerivation {
        name = "data";
        src = templates;
        buildPhase = ''
          ${nixpkgs.tailwindcss}/bin/tailwindcss -i src/input.css -o public/output.css
        '';
        installPhase = ''
          mkdir -p $out
          cp -r $src/public $src/templates $out/
        '';
      };
      bin = crane.buildPackage (commonArgs // { cargoArtifacts = auth-deps; });
    in
    nixpkgs.writeShellScriptBin "auth" ''
      export PUBLIC_DIR="${data}/public"
      export TEMPLATES_DIR="${data}/templates"
      ${bin}/bin/auth
    '';
}
