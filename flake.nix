{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [
            (import ./nix/overlays/hdf5.nix)
          ];
        };

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.75";
          date = "2023-12-28";
          sha256 = "SXRtAuO4IqNOQq+nLbrsDFbVk+3aVA8NNpSZsKlVH/8=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        ws_cargo_toml = builtins.readFile ./Cargo.toml;
        ws_cargo = builtins.fromTOML ws_cargo_toml;

        version = ws_cargo.workspace.package.version;
        git_revision = self.shortRev or self.dirtyShortRev;

        hdf5-joined = pkgs.symlinkJoin {
          name = "hdf5";
          paths = with pkgs; [hdf5 hdf5.dev];
        };
        nativeBuildInputs = with pkgs; [cmake flatbuffers hdf5-joined perl tcl pkg-config];
        buildInputs = with pkgs; [openssl cyrus_sasl hdf5-joined];

        lintingRustFlags = "-D unused-crate-dependencies";
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [toolchain.toolchain];
          buildInputs = buildInputs;

          packages = with pkgs; [
            # Newer version of nix is required to use `dirtyShortRev`
            nix

            # Code formatting tools
            alejandra
            treefmt

            # Container image management
            skopeo
          ];
          RUSTFLAGS = lintingRustFlags;
          HDF5_DIR = "${hdf5-joined}";
        };

        packages =
          {
            test = naersk'.buildPackage {
              mode = "test";
              src = ./.;

              nativeBuildInputs = nativeBuildInputs;
              buildInputs = buildInputs;
              HDF5_DIR = "${hdf5-joined}";

              # Ensure detailed test output appears in nix build log
              cargoTestOptions = x: x ++ ["1>&2"];
            };
          }
          // import ./events-to-histogram {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./kafka-daq-report {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./run-simulator {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./simulator {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./stream-to-file {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs hdf5-joined;}
          // import ./trace-archiver {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs hdf5-joined;}
          // import ./trace-archiver-tdengine {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./trace-reader {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./trace-to-events {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;};
      }
    );
}
