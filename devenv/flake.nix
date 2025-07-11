{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-unstable";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
    systems = {
      url = "github:nix-systems/default";
    };
  };
  outputs =
    {
      nixpkgs,
      rust-overlay,
      systems,
      ...
    }:
    {
      devShells = nixpkgs.lib.genAttrs (import systems) (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              (pkgs.rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; })
              pkgs.pkg-config
            ];
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );
    };
}
