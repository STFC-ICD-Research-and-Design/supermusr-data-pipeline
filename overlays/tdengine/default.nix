{
  nixpkgs,
  stdenv,
  fetchFromGitHub
}: stdenv.mkDerivation {
  name = "TDengine-client";
  version = "3.0.4.2";

  src = fetchFromGitHub {
    owner = "taosdata";
    repo = "TDEngine";
    rev = "ver-3.0.4.2";
    hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
  };

  dontUseCmakeConfigure=true;

  nativeBuildInputs = with nixpkgs; [
    cacert
    git
    pkg-config
    xz
    jansson
    cmake
  ];

  buildPhase = ''
    ./build.sh
  '';
  
  installPhase = ''
    make install
  '';
}