{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        gitu = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          doCheck = false;
          buildInputs = with pkgs;
            [ openssl pkg-config ] ++ nixpkgs.lib.optionals stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.Security ];
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
