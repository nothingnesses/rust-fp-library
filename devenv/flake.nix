{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix-monthly = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
          inputs.flake-parts.flakeModules.easyOverlay
        ];
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
          let
            rustToolchain =
              with inputs.fenix.packages.${pkgs.stdenv.hostPlatform.system};
              combine [
                stable.clippy
                stable.rustc
                stable.cargo
                inputs.fenix-monthly.packages.${pkgs.stdenv.hostPlatform.system}.latest.rustfmt
                stable.rust-src
              ];

            treefmtEval = inputs.treefmt-nix.lib.evalModule pkgs {
              # Cargo.toml lives at the repo root (one level above devenv/).
              # This tells treefmt where the project root is so it formats the
              # entire repository, not just the devenv/ subdirectory.
              projectRootFile = "Cargo.toml";
              programs = {
                nixfmt.enable = true;
                rustfmt = {
                  enable = true;
                  # Use the nightly rustfmt from fenix rather than the
                  # nixpkgs rustfmt, so that unstable options in
                  # rustfmt.toml are supported.
                  package = rustToolchain;
                };
                prettier = {
                  enable = true;
                  includes = [
                    "*.md"
                    "*.yml"
                    "*.yaml"
                  ];
                };
              };
              settings.formatter.tombi = {
                command = "${inputs.nixpkgs-unstable.legacyPackages.${system}.tombi}/bin/tombi";
                options = [
                  "format"
                  "--offline"
                ];
                includes = [ "*.toml" ];
              };
            };

            pre-commit-check = inputs.git-hooks.lib.${system}.run {
              src = ./..;
              hooks = {
                treefmt = {
                  enable = true;
                  package = treefmtEval.config.build.wrapper;
                  # Format in-place and re-stage, so commits succeed in
                  # one pass instead of requiring a double-commit.
                  entry =
                    let
                      script = pkgs.writeShellScript "treefmt-and-stage" ''
                        ${treefmtEval.config.build.wrapper}/bin/treefmt --no-cache "$@"
                        git add "$@"
                      '';
                    in
                    "${script}";
                };
                # clippy and doc delegate to just recipes (single source of
                # truth for flags like -D warnings and the unicode check).
                # They run on pre-push rather than pre-commit because
                # pre-commit stashes unstaged changes, which breaks
                # whole-project tools when only some files are staged.
                clippy = {
                  enable = true;
                  entry = "${pkgs.just}/bin/just clippy --workspace --all-features";
                  pass_filenames = false;
                  always_run = true;
                  stages = [ "pre-push" ];
                };
                cargo-doc = {
                  enable = true;
                  entry = "${pkgs.just}/bin/just doc --workspace --all-features --no-deps";
                  pass_filenames = false;
                  always_run = true;
                  stages = [ "pre-push" ];
                };
              };
            };
          in
          {
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

            packages.rustToolchain = rustToolchain;

            formatter = treefmtEval.config.build.wrapper;

            checks = {
              formatting = treefmtEval.config.build.check self'.self;
              inherit pre-commit-check;
            };

            devShells.default = pkgs.mkShell {
              packages = [
                pkgs.bashInteractive
                config.packages.rustToolchain
                pkgs.openssl
                pkgs.pkg-config
                pkgs.cargo-deny
                pkgs.cargo-edit
                pkgs.bacon
                pkgs.rust-analyzer
                pkgs.gh
                pkgs.just
                pkgs.rust-script
                pkgs.python3
                # For criterion.rs
                pkgs.gnuplot
                # For link checking in markdown
                pkgs.lychee
              ];

              env = {
                # Required by rust-analyzer
                RUST_SRC_PATH = "${config.packages.rustToolchain}/lib/rustlib/src/rust/library";
              };

              inherit (pre-commit-check) shellHook;
            };
          };
      }
    );
}
