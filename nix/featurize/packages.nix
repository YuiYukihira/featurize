{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;

  cargoArtifacts = cell.packages.featurize-deps;
in
with cell.args; rec {
  featurize-deps = crane.buildDepsOnly commonArgs;

  featurize =
    let
      templates = std.incl inputs.self [
        (inputs.self + /templates)
        (inputs.self + /public)
        (inputs.self + /src/input.css)
        (inputs.self + /tailwind.config.js)
      ];
      bin =
        crane.buildPackage (commonArgs // { cargoArtifacts = featurize-deps; });
    in
    nixpkgs.stdenv.mkDerivation {
      name = "featurize";
      version = commonArgs.version;

      src = templates;

      buildPhase = ''
        ${nixpkgs.tailwindcss}/bin/tailwindcss -i src/input.css -o public/output.css
      '';

      installPhase = ''
        mkdir -p $out/{bin,share}
        cp ${bin}/bin/featurize $out/bin/featurize
        cp -r {public,templates} $out/share/
      '';

      meta.mainProgram = "featurize";
    };
}
