with import <nixpkgs> {};

{
  stdenv
}: rec {
  default = stdenv.mkDerivation {
    name = "TDengine-client";
    version = "3.0.4.2";

    src = fetchurl {
      url = "https://www.taosdata.com/assets-download/3.0/TDengine-client-${default.version}-Linux-x64.tar.gz";
      hash = "sha256-7qshbjOKF9fHpaT7UNAUlQAMtWh1BN/GSwKe2/k3VF0=";
    };
    src = fetchFromGitHub {
      owner = "taosdata";
      repo = "TDEngine";
      rev = "ver-${default.version}";
      hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
    };

    nativeBuildInputs = [ cmake ];
    buildInputs = [ ];

    unpackPhase = ''
      ls;
    '';

    buildPhase = ''
    '';

    installPhase = ''
      sudo make install
    '';
  };
}