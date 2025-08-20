{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-unstable,
    flake-utils,
    fenix,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        pkgs-unstable = (import nixpkgs-unstable) {
          inherit system;
        };

        nativeRustToolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.88";
          date = "2025-06-26";
          sha256 = "Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
        };

        wasm32RustToolchain = fenix.packages.${system}.targets.wasm32-unknown-unknown.toolchainOf {
          channel = "1.88";
          date = "2025-06-26";
          sha256 = "Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
        };

        rustToolchain = with fenix.packages.${system};
          combine [
            nativeRustToolchain.toolchain
            wasm32RustToolchain.toolchain
          ];

        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = workspaceCargo.workspace.package.version;
        gitRevision = self.shortRev or self.dirtyShortRev;

        nativeBuildInputs = with pkgs; [
          cmake
          flatbuffers
          perl
          tcl
          pkg-config
          clang
        ];
        buildInputs = with pkgs; [
          openssl
          cyrus_sasl
        ];

        lintingRustFlags = "-D unused-crate-dependencies";
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [rustToolchain];
          buildInputs = buildInputs;

          packages = with pkgs; [
            # Code formatting tools
            alejandra
            treefmt
            mdl

            # Dependency auditing
            cargo-deny

            # Container image management
            skopeo

            # Documentation tools
            adrs

            # Additional toolchain components for trace-viewer
            pkgs-unstable.cargo-leptos
            dart-sass
            binaryen
          ];

          RUSTFLAGS = lintingRustFlags;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        packages =
          import ./diagnostics {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./digitiser-aggregator {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./nexus-writer {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./simulator {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-reader {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-telemetry-exporter {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-to-events {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-viewer {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-viewer-tui {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;};
      }
    );
}
