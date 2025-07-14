{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

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
        };

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.87";
          date = "2025-05-15";
          sha256 = "KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = workspaceCargo.workspace.package.version;
        gitRevision = self.shortRev or self.dirtyShortRev;

        wasm-bindgen-cli = pkgs.callPackage "${nixpkgs}/pkgs/by-name/wa/wasm-bindgen-cli/package.nix" {
          version = "0.2.95";
          hash = "sha256-prMIreQeAcbJ8/g3+pMp1Wp9H5u+xLqxRxL+34hICss=";
          cargoHash = "sha256-6iMebkD7FQvixlmghGGIvpdGwFNLfnUcFke/Rg8nPK4=";
        };


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

        wasm-toolchain = fenix.packages.${system}.targets.wasm32-unknown-unknown.toolchainOf {
          channel = "1.87";
          date = "2025-05-15";
          sha256 = "KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU=";
        };
        
        combined-toolchain-derivation = with fenix.packages.${system}; combine [
          toolchain.toolchain
          wasm-toolchain.toolchain
        ];
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [ combined-toolchain-derivation ];
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

            # Server
            trunk
            cargo-leptos
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
          // import ./trace-server {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-telemetry-exporter {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-to-events {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;}
          // import ./trace-viewer {inherit pkgs naersk' version gitRevision nativeBuildInputs buildInputs;};
      }
    );
}
