{pkgs ? import <nixpkgs> {overlays = [(import ./nix/overlays/hdf5.nix)];}}: let
  hdf5-joined = pkgs.symlinkJoin {
    name = "hdf5";
    paths = with pkgs; [hdf5 hdf5.dev];
  };
in
  pkgs.mkShell {
    # nativeBuildInputs is usually what you want -- tools you need to run
    nativeBuildInputs = with pkgs; [
      rustup
      rustc
      cargo
      rustfmt
      clippy
      hdf5-joined
      pkg-config
      openssl
      cmake
      cyrus_sasl
    ];

    HDF5_DIR = "${hdf5-joined}";

    RUST_LOG = "full";
  }
