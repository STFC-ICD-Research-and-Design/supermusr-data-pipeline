with import <nixpkgs> {};

{
  stdenv
}: rec {
  packages.x86_64-linux.default = stdenv.mkDerivation {
    name = "TDengine-client";
    version = "3.0.4.2";

    src = fetchFromGitHub {
      owner = "taosdata";
      repo = "taos-tools";
      rev = "v${version}";
    };

    nativeBuildInputs = [ cmake ];
    buildInputs = [ ];

    buildPhase = ''
      echo "Hello Once"
    '';
      #mkdir build
      #echo "Hello Twice"
      #cd build
      #cmake ..
      #echo "Hello Thrice"
      #make
      #echo "Hello"

    installPhase = ''
      sudo make install
    '';
  };
}