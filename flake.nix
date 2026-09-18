{
  description = "Type-level left projections for tuple and product types.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-parts.url = "github:hercules-ci/flake-parts";

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = { system, ... }:
        let
          pkgs = inputs.nixpkgs.legacyPackages.${system}.extend (
            inputs.rust-overlay.overlays.default
          );

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "clippy"
              "rust-src"
              "rustfmt"
            ];
          };

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            pname = "tuple-projections";
            version = "0.1.0";
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          package = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          treefmt = {
            projectRootFile = "flake.nix";

            programs = {
              nixfmt.enable = true;
              rustfmt = {
                enable = true;
                package = rustToolchain;
              };
              taplo.enable = true;
            };
          };

          packages.default = package;

          checks = {
            build = package;

            test = craneLib.cargoTest (commonArgs // {
              inherit cargoArtifacts;
              cargoTestExtraArgs = "--workspace";
            });

            clippy = craneLib.cargoClippy (commonArgs // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--workspace --all-targets --all-features -- -D warnings";
            });

            audit = craneLib.cargoAudit {
              inherit src;
              advisory-db = inputs.rust-advisory-db;
            };
          };

          devShells.default = pkgs.mkShell {
            packages = [
              rustToolchain
              pkgs.cargo-audit
              pkgs.cargo-nextest
              pkgs.rust-analyzer
            ];
          };
        };
    };
}
