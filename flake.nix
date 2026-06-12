{
  description = "Persista flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  };
  outputs =
    { self, nixpkgs, ... }:
    let
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          persista = pkgs.callPackage ./package.nix { };
        in
        {
          inherit persista;
          default = persista;
        }
      );
      nixosModules.persista = import ./module.nix;
      nixosModules.default = self.nixosModules.persista;
    };
}
