{
  nixpkgs,
  stdenv,
  fetchFromGitHub
}: stdenv.mkDerivation {
  name = "TDengine-client";
  version = "3.1.1.7";

  src = fetchFromGitHub {
    owner = "taosdata";
    repo = "TDEngine";
    rev = "ver-3.1.1.0";
    hash = "sha256-Aua3i7YIo4U56KaOZuLJEAc5fIxG5Pux9LM2TATwl40=";
  };

  nativeBuildInputs = with nixpkgs; [
    cacert
    git
    pkg-config
    xz
    libuv
    jansson
    cmake
    apr
    aprutil
    curl
    geos
  ];

  #buildPhase = ''
  #  bash ./build.sh
  #'';
  buildPhase = ''
    mkdir debug
    cd debug
    cmake .. -DBUILD_TOOLS=false
    make
  '';
  
  installPhase = ''
    make install
  '';
}