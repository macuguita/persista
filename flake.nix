{
  description = "Persista API flake";

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
          dev-pg = pkgs.callPackage ./nix/dev-pg { };
          stop-pg = pkgs.callPackage ./nix/stop-pg { };
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
              dev-pg # Script that starts postres database
              stop-pg # Script that stops previous postgres database
            ];

            shellHook = ''
              export DB_URL="postgres://$USER@localhost:54329/persista"
              export JWT_SECRET="dev-jwt-secret"
              export ADMIN_SECRET="dev-admin-secret"

              echo
              echo "DB_URL=$DB_URL"
              echo
            '';
          };
        }
      );
    };
}
