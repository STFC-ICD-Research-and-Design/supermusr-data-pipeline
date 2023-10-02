{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.72";
          sha256 = "dxE7lmCFWlq0nl/wKcmYvpP9zqQbBitAQgZ1zx9Ooik=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        ws_cargo_toml = builtins.readFile ./Cargo.toml;
        ws_cargo = builtins.fromTOML ws_cargo_toml;

        version = ws_cargo.workspace.package.version;
        git_revision = self.shortRev or self.dirtyShortRev;

        nativeBuildInputs = with pkgs; [ cmake flatbuffers hdf5 perl tcl ];

      in rec {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [ toolchain.toolchain ];
          packages = with pkgs; [ nix skopeo ];
        };

        packages = {
          events-to-histogram = import ./events-to-histogram { inherit pkgs naersk' version git_revision nativeBuildInputs; };
          kafka-daq-report = import ./kafka-daq-report { inherit pkgs naersk' version git_revision nativeBuildInputs; };
          simulator = import ./simulator { inherit pkgs naersk' version git_revision nativeBuildInputs; };
          stream-to-file = import ./stream-to-file { inherit pkgs naersk' version git_revision nativeBuildInputs; };
          trace-archiver = import ./trace-archiver { inherit pkgs naersk' version git_revision nativeBuildInputs; };
          trace-to-events = import ./trace-to-events { inherit pkgs naersk' version git_revision nativeBuildInputs; };

          fmt = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            mode = "fmt";
          };

          clippy = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            mode = "clippy";
          };

          test = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            mode = "test";
            # Ensure detailed test output appears in nix build log
            cargoTestOptions = x: x ++ ["1>&2"];
          };
        };
      }
    );
}
