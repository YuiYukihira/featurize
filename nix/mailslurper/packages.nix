{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  mailslurper = nixpkgs.buildGoModule {
    pname = "mailslurper";
    version = "1.15.0";

    src = nixpkgs.fetchFromGitHub {
      owner = "mailslurper";
      repo = "mailslurper";
      rev = "5d7ca4938115e52d04aee29277c3d3a8981db21d";
      hash = "sha256-H74YdfFb3+rDQnlmYmJjp/l53EfbtHuNNL1eXXJKWNs=";
    };

    vendorHash = "sha256-/SZC1CGQL8RW21416OalmvZGfR+V26ole1tb4OhEyPw=";
  };
}
