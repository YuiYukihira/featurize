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
      bin = crane.buildPackage (commonArgs // { cargoArtifacts = auth-deps; });
    in
    nixpkgs.stdenv.mkDerivation {
      name = "auth";
      version = commonArgs.version;

      src = templates;

      buildPhase = ''
        ${nixpkgs.tailwindcss}/bin/tailwindcss -i src/input.css -o public/output.css
      '';

      installPhase = ''
        mkdir -p $out/{bin,share}
        cp ${bin}/bin/auth $out/bin/auth
        cp -r {public,templates} $out/share/
      '';

      meta.mainProgram = "auth";
    };
}
