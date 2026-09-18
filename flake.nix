{
  description = "Type-level left projections for tuple and product types.";

  inputs = {
    # Nix and flake composition.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-parts.url = "github:hercules-ci/flake-parts";

    # Reusable Rust development tools and the rust-overlay capability.
    nix-devtools = {
      url = "github:alekseysidorov/nix-devtools";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
      inputs.treefmt-nix.follows = "treefmt-nix";
    };

    crane.url = "github:ipetkov/crane";

    rust-advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake
      {
        inherit inputs;
        # Keep provider-owned inputs available to composed flake modules without
        # turning them into a broad ambient dependency channel.
        specialArgs = {
          localInputs = inputs;
        };
      }
      (
        { ... }:
        let
          inherit (inputs.nixpkgs) lib;

          # Reuse nix-devtools' public overlay so its Rust toolchain capability
          # has one owner and is available to this flake's own package universe.
          defaultOverlay = inputs.nix-devtools.overlays.default;
        in
        {
          systems = lib.systems.flakeExposed;

          imports = [
            inputs.treefmt-nix.flakeModule
            inputs.nix-devtools.flakeModule
          ];

          flake = {
            # Expose the same overlay consumed internally for downstream users.
            overlays.default = defaultOverlay;
          };

          perSystem =
            { system, ... }:
            let
              # Use one package universe, extended through the public overlay;
              # this keeps rust-bin and all check tooling on the same pkgs set.
              pkgs = inputs.nixpkgs.legacyPackages.${system}.extend inputs.self.overlays.default;

              rustToolchain = pkgs.rust-bin.stable.latest.default.override {
                extensions = [
                  "clippy"
                  "rust-src"
                  "rustfmt"
                ];
              };

              craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
              src = craneLib.cleanCargoSource ./.;

              # Keep dependency compilation separate so build, test and clippy
              # checks reuse the same Cargo artifacts.
              commonArgs = {
                inherit src;
                pname = "tuple-projections";
                version = "0.1.0";
                strictDeps = true;
                # trybuild diagnostics vary between isolated Cargo environments;
                # the cases still have to fail, while local Cargo checks compare snapshots.
                preCheck = "export TRYBUILD=overwrite";
              };

              cargoArtifacts = craneLib.buildDepsOnly commonArgs;

              package = craneLib.buildPackage (
                commonArgs
                // {
                  inherit cargoArtifacts;
                }
              );

              # Keep semver compatibility as an explicit runnable check. It is
              # intentionally a package rather than a default flake check because
              # the registry baseline exists only after the crate is published.
              semverCheck = pkgs.writeShellApplication {
                name = "check-cargo-semver";
                runtimeInputs = [
                  rustToolchain
                  pkgs.cargo-semver-checks
                ];
                text = ''
                  exec cargo semver-checks --workspace "$@"
                '';
              };
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

              packages = {
                default = package;
                check-cargo-semver = semverCheck;
              };

              checks = {
                build = package;

                test = craneLib.cargoTest (
                  commonArgs
                  // {
                    inherit cargoArtifacts;
                    cargoTestExtraArgs = "--workspace";
                  }
                );

                clippy = craneLib.cargoClippy (
                  commonArgs
                  // {
                    inherit cargoArtifacts;
                    cargoClippyExtraArgs = "--workspace --all-targets --all-features -- -D warnings";
                  }
                );

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
                  pkgs.cargo-semver-checks
                  pkgs.rust-analyzer
                ];
              };
            };
        }
      );
}
