{ pkgs, ... }:
let
  vesperDoctor = pkgs.callPackage ./packages/vesper-doctor.nix { };
in
{
  home.packages = [ vesperDoctor ];
}
