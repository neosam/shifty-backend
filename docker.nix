{ pkgs ? import <nixpkgs> {} }:
let
  shiftyService = import ./build.nix { inherit pkgs; };
in pkgs.dockerTools.buildImage {
  name = "shifty-backend";
  tag = "0.1";

  copyToRoot = pkgs.buildEnv {
    name = "terminal-tools";
    paths = [ pkgs.bash pkgs.coreutils pkgs.nix ];
  };
  config = {
    Cmd = [ "${shiftyService}/bin/app" ];
  };
}
