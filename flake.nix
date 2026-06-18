{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          persista = pkgs.callPackage ./nix/package.nix { };
        in
        {
          inherit persista;
          default = persista;
        }
      );

      nixosModules.persista = import ./nix/module.nix;
      nixosModules.default = self.nixosModules.persista;

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              nixd
              nixfmt
              cargo
              rustc
              rustfmt
              pre-commit
              clippy
              sqlx-cli
              postgresql
            ];
          };
        }
      );
    };
}
