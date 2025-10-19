{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/v0.12.0";
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
          rustVersion = "1.82.0";
          packageFun = import ./Cargo.nix;
        };

      in rec {
        packages = {
          clap4shell = (rustPkgs.workspace.clap4shell {});
          default = packages.clap4shell;
        };
        defaultPackage = packages.default;
        apps = rec {
          clap4shell = { type = "app"; program = "${packages.default}/bin/clap4shell"; };
          default = clap4shell;
          cargo2nix = cargo2nix.apps.${system}.default;
        };
        devShell.default = rustPkgs.workspaceShell {};
      }
    )// {
    cross."x86_64-linux".packages."aarch64-linux" =
    let
      pkgs = import nixpkgs {
        overlays = [ cargo2nix.overlays.default ];
        localSystem = "x86_64-linux";
        crossSystem.config = "aarch64-unknown-linux-gnu";
      };
      rustPkgs = pkgs.rustBuilder.makePackageSet {
        rustVersion = "1.61.0";
        packageFun = import ./Cargo.nix;
        target = "aarch64-unknown-linux-gnu";
      };
    in
    {
      clap4shell = (rustPkgs.workspace.clap4shell { });
    };
  };
}
