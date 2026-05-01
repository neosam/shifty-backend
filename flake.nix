{
  description = "Shifty Backend - Shiftyplan Service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    openspec.url = "github:Fission-AI/OpenSpec";
    gsd.url = "github:neosam/gsd-flake";
  };

  outputs = { self, nixpkgs, flake-utils, openspec, gsd }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          # Backend mit mock_auth (default)
          default = pkgs.callPackage ./default.nix { 
            inherit pkgs;
            features = [ "mock_auth" ];
          };
          
          # Backend mit mock_auth
          backend-mock = pkgs.callPackage ./default.nix {
            inherit pkgs;
            features = [ "mock_auth" "" ];
          };
          
          # Backend mit OIDC
          backend-oidc = pkgs.callPackage ./default.nix {
            inherit pkgs;
            features = [ "oidc" "json_logging" ];
          };
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            cargo-watch
            clippy
            rust-analyzer
            sqlx-cli
            sqlite
            nodejs
            pkg-config
            
            openspec.packages.${system}.default
            gsd.packages.${system}.default
          ];
        };
      }
    ) // {
      # NixOS Module (system-unabhängig)
      nixosModules.default = import ./module.nix;
    };
}
