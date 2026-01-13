{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      top@{
        config,
        withSystem,
        moduleWithSystem,
        ...
      }:
      {
        imports = [
          # Optional: use external flake logic, e.g.
          # inputs.foo.flakeModules.default
          inputs.flake-parts.flakeModules.easyOverlay
        ];
        # flake = {
        #   # Put your original flake attributes here.
        # };
        # systems for which you want to build the `perSystem` attributes
        systems = import inputs.systems;
        perSystem =
          {
            self',
            pkgs,
            config,
            lib,
            final,
            system,
            ...
          }:
          {
            # Recommended: move all package definitions here.
            # e.g. (assuming you have a nixpkgs input)
            # packages.foo = pkgs.callPackage ./foo/package.nix { };
            # packages.bar = pkgs.callPackage ./bar/package.nix {
            #   foo = config.packages.foo;
            # };

            formatter = pkgs.nixfmt;

            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;
              config.allowUnfree = true;
              overlays = [
                (final: _prev: {
                  unstable = import inputs.nixpkgs-unstable {
                    inherit (final) system;
                    config.allowUnfree = true;
                  };
                })
              ];
            };

            overlayAttrs = {
              inherit (config.packages) rustToolchain;
            };

            packages.rustToolchain =
              with inputs.fenix.packages.${pkgs.stdenv.hostPlatform.system};
              combine [
                stable.clippy
                stable.rustc
                stable.cargo
                stable.rustfmt
                stable.rust-src
              ];

            devShells.default = pkgs.mkShell {
              # Alias for `nativeBuildInputs`
              # https://discourse.nixos.org/t/difference-between-buildinputs-and-packages-in-mkshell/60598/10
              packages = [
                pkgs.bashInteractive
                config.packages.rustToolchain
                pkgs.openssl
                pkgs.pkg-config
                pkgs.cargo-deny
                pkgs.cargo-edit
                pkgs.bacon
                pkgs.rust-analyzer
                pkgs.just
                # For criterion.rs
                pkgs.gnuplot
              ];

              env = {
                # Required by rust-analyzer
                RUST_SRC_PATH = "${config.packages.rustToolchain}/lib/rustlib/src/rust/library";
              };
            };
          };
      }
    );
}
