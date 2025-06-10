{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";

  outputs =
    { self, nixpkgs }:
    {
      packages =
        nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
          ]
          (system: rec {
            impala = nixpkgs.legacyPackages.${system}.callPackage ./package.nix { };
            default = impala;
          });
    };
}
