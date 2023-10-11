{
  self,
  nixpkgs,
  stdenv,
  fetchFromGitHub
}: 
stdenv.mkDerivation {
  name = "TDengine-client";
  version = "3.0.4.2";

  src = fetchFromGitHub {
    owner = "taosdata";
    repo = "TDEngine";
    rev = "ver-${self.version}";
    hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
  };

  dontUseCmakeConfigure=true;

  nativeBuildInputs = with nixpkgs; [
    cjson
    lz4
    git
    cmake
  ];

  buildPhase = ''
    mkdir debug
    cd debug
    cmake .. -DBUILD_TOOLS=false -DBUILD_HTTP=true
    ls
    make
    ls -f
  '';
  
  installPhase = ''
    make install
  '';
}