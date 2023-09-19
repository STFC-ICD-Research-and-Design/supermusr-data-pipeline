{
  inputs = {
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
          sha256 = "Q9UgzzvxLi4x9aWUJTn+/5EXekC98ODRU1TwhUs9RnY=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        ws_cargo_toml = builtins.readFile ./Cargo.toml;
        ws_cargo = builtins.fromTOML ws_cargo_toml;

        version = ws_cargo.workspace.package.version;
        git_revision = self.shortRev or self.dirtyShortRev;

        nativeBuildInputs = with pkgs; [ cmake pkg-config flatbuffers ];
        buildInputs = with pkgs; [ openssl ];

      in rec {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [ toolchain.toolchain ];
          buildInputs = buildInputs;
          packages = with pkgs; [ nix skopeo ];
        };

        packages = {
          events-to-histogram = import ./events-to-histogram { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };
          kafka-daq-report = import ./kafka-daq-report { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };
          simulator = import ./simulator { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };
          stream-to-file = import ./stream-to-file { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };
          trace-archiver = import ./trace-archiver { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };
          trace-to-events = import ./trace-to-events { inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs; };

          fmt = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            mode = "fmt";
          };

          clippy = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            mode = "clippy";
          };

          test = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
            mode = "test";
            # Ensure detailed test output appears in nix build log
            cargoTestOptions = x: x ++ ["1>&2"];

            AWS_ACCESS_KEY_ID = "minioadmin";
            AWS_SECRET_ACCESS_KEY = "minioadmin";
          };
        };
      }
    );
}
