{
  nixpkgs,
  stdenv,
  fetchFromGitHub
}: rec {
  default = stdenv.mkDerivation {
    name = "TDengine-client";
    version = "3.0.4.2";

    src = fetchFromGitHub {
      owner = "taosdata";
      repo = "TDEngine";
      rev = "ver-${default.version}";
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
  };
}