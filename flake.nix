{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};

        pkgs = import nixpkgs {
          inherit system;
          packages.${system}.default =
            fenix.packages.${system}.default.toolchain;
        };

        gitu = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          doCheck = false;
          buildInputs = with pkgs;
            [ openssl pkg-config ] ++ nixpkgs.lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
            ];
        };
      in {
        checks = { inherit gitu; };

        packages.default = gitu;
        apps.default = flake-utils.lib.mkApp { drv = gitu; };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          inputsFrom = [ gitu ];
          packages = with pkgs; [ clippy rust-analyzer rustfmt ];
        };
      });
}
