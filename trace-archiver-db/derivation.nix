{ stdenv }:
stdenv.mkDerivation rec {
  name = "TDengine-client-${version}";
  version = "3.0.4.2";

  src = fetchFromGitHub {
    owner = "taosdata";
    repo = "taos-tools";
    rev = ${version};
  };

  nativeBuildInputs = [ ];
  buildInputs = [ ];

  buildPhase = ''
    mkdir build
    cd build
    cmake ..
    make
  '';

  installPhase = ''
    sudo make install
  '';
}