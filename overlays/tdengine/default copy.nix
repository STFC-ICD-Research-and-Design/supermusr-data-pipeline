{
  nixpkgs,
  stdenv,
  fetchFromGitHub
}:
  let
    version = "3.0.4.2";
  in stdenv.mkDerivation {
  name = "TDengine-client";
  version = version;

  src = fetchFromGitHub {
    owner = "taosdata";
    repo = "TDEngine";
    rev = "ver-${version}";
    hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
  };

  dontUseCmakeConfigure=true;

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