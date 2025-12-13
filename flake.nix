{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "aarch64-linux" "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { config, pkgs, system, lib, inputs', self', ... }:
        let
          toolchain = with inputs.fenix.packages.${system};
            combine [
              latest.rustc
              latest.cargo
              latest.clippy
              latest.rust-analysis
              latest.rust-src
              latest.rustfmt
            ];

          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;
          pestFilter = path: _type: builtins.match ".*pest$" path != null;
          pestOrCargo = path: type: (pestFilter path type) || (craneLib.filterCargoSources path type);
          common-build-args = {
            src = lib.cleanSourceWith {
              src = ./.;
              filter = pestOrCargo;
              name = "source";
            };
          };
          deps-only = craneLib.buildDepsOnly ({ } // common-build-args);

          pre-commit-check = inputs.git-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt = {
                enable = true;
                packageOverrides = {
                  cargo = toolchain;
                  rustfmt = toolchain;
                };
              };
              clippy = {
                enable = true;
                packageOverrides = {
                  cargo = toolchain;
                  clippy = toolchain;
                };
              };
            };
          };

          packages = {
            default = packages.gitu;
            gitu = craneLib.buildPackage (common-build-args // {
              doCheck = false;
              buildInputs = with pkgs;
                [ openssl pkg-config ] ++ pkgs.lib.optionals stdenv.isDarwin [
                  libiconv
                  darwin.apple_sdk.frameworks.Security
                  darwin.apple_sdk.frameworks.AppKit
                ];
            });
          };

          checks = {
            clippy = craneLib.cargoClippy ({
              cargoArtifacts = deps-only;
              cargoClippyExtraArgs = "--all-features -- --deny warnings";
            } // common-build-args);

            rust-fmt = craneLib.cargoFmt
              ({ inherit (common-build-args) src; } // common-build-args);

            rust-tests = craneLib.cargoNextest ({
              cargoArtifacts = deps-only;
              partitions = 1;
              partitionType = "count";
            } // common-build-args);
          };

        in rec {
          inherit packages checks;

          devShells.default = pkgs.mkShell {
            inherit (pre-commit-check) shellHook;
            inputsFrom = [ packages.gitu ];
            nativeBuildInputs = with pkgs; [
              toolchain
              git-cliff
              cargo-nextest
              cargo-deny
            ];
          };
        };
    };
}
