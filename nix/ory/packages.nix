{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  kratos = nixpkgs.buildGoModule {
    pname = "kratos";
    version = "1.1.0";

    tags = [ "sqlite" "json1" "hsm" ];
    subPackages = [ "." ];

    src = nixpkgs.fetchFromGitHub {
      owner = "ory";
      repo = "kratos";
      rev = "0f81b768499a5c9240b34761e90a712f97003c55";
      hash = "sha256-O2gY3mDL4DUPyK5P37F+R1/Vr3AD+o7/KJ9LJGqRJig=";
    };

    vendorHash = "sha256-1lquBtlRfJEMmdnYTIerWB1XO2zQhthiWBJVRP37iOI=";
  };

  hydra = nixpkgs.buildGoModule {
    pname = "hydra";
    version = "2.2.0";

    tags = [ "sqlite" "json1" "hsm" ];
    subPackages = [ "." ];

    src = nixpkgs.fetchFromGitHub {
      owner = "ory";
      repo = "hydra";
      rev = "fcaace45b4f46c2346e1927cc0cbfc78bed0ab34";
      hash = "sha256-H+8TCqEDX5SU8HbH8kQj/NvBZ+inwJPjnMvaMuoSpsU=";
    };

    vendorHash = "sha256-/ED/5a8IzKLc8fmV45uWhP96L09Xpl7RwXXROHwWnKc=";
  };
}
