{pkgs, ...}: {
  packages = with pkgs; [
    # Rust toolchain
    rustup
    clang
    cmake
    flatbuffers
    pkg-config

    # Code formatting tools
    alejandra
    treefmt
    mdl

    # Dependency auditing
    cargo-deny

    # Additional toolchain components for trace-viewer
    cargo-leptos
    binaryen
    dart-sass
  ];

  env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
