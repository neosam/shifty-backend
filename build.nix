{ pkgs ? import <nixpkgs> {}, features ? [] }:
let
  specificPkgs = import (pkgs.fetchFromGitHub {
    owner = "NixOS";
    repo = "nixpkgs";
    rev = "57610d2f8f0937f39dbd72251e9614b1561942d8";
    sha256 = "sha256-yZKhxVIKd2lsbOqYd5iDoUIwsRZFqE87smE2Vzf6Ck0=";
  }) {};
  rustPlatform = specificPkgs.rustPlatform;
in
  rustPlatform.buildRustPackage {
    pname = "shifty-service";
    version = "0.1";
    src = ./.;
    buildFeatures = features;

    cargoHash = "sha256-sTKupn3HMBf3lumCu1RUkzutc+RUNpuqEyGR2BMxAso=";
  }
