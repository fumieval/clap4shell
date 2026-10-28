{
  description = "rich getopts replacement for shell scripts";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      mkPackage =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "clap4shell";
          inherit version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          meta = {
            description = "rich getopts replacement for shell scripts";
            homepage = "https://github.com/fumieval/clap4shell";
            license = pkgs.lib.licenses.mit;
            mainProgram = "clap4shell";
          };
        };
    in
    {
      packages = forAllSystems (pkgs: {
        clap4shell = mkPackage pkgs;
        default = mkPackage pkgs;
      });
      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${mkPackage pkgs}/bin/clap4shell";
        };
      });
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ (mkPackage pkgs) ];
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
          ];
        };
      });
    };
}
