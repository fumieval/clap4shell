{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.61.0";
          packageFun = import ./Cargo.nix;
        };

      in rec {
        packages = {
          clap4shell = (rustPkgs.workspace.clap4shell {}).bin;
          default = packages.clap4shell;
        };
        defaultPackage = packages.default;
        apps = rec {
          clap4shell = { type = "app"; program = "${packages.default}/bin/clap4shell"; };
          default = clap4shell;
        };
      }
    );
}