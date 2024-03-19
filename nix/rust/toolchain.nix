{ inputs, cell }: {
  rust = (inputs.nixpkgs.extend
    inputs.rust-overlay.overlays.default).rust-bin.fromRustupToolchainFile
    (inputs.self + /rust-toolchain.toml);
}
