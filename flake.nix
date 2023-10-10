{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; overlays = [ tdengineOverlay ]; };
        inherit system;
      });

      tdengineOverlay = final: prev: {
          tdengine = prev.callPackage ./overlays/tdengine/default.nix {};
      };
    in
    {
      devShells = forAllSystems ({ pkgs, system }: {
        default = pkgs.mkShell {
          packages = (with pkgs; [
            flatbuffers
            rustup
            cmake
            ninja
            zlib
            zstd
            rdkafka
            tdengine
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
        };
      });
    };
}
