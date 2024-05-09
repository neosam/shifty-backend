{ pkgs ? import <nixpkgs> {} }:
let
  rustPlatform = pkgs.rustPlatform;
in
  rustPlatform.buildRustPackage {
    pname = "shifty-service";
    version = "0.1";
    src = ./.;
    cargoHash = "sha256-bgtX30TGRlBjCZ8qbqNgovsZrZqJ9kEGlv/qv6T5uZA=";
  }
