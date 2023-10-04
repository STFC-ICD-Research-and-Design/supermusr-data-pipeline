{
  description = "A very basic flake";

  outputs = { self, nixpkgs, lib, stdenv, fetchFromGitHub, cmake, zlib }: {

    packages.x86_64-linux.default = stdenv.mkDerivation {
      name = "TDengine-client-${version}";
      version = "3.0.4.2";

      src = fetchFromGitHub {
        owner = "taosdata";
        repo = "taos-tools";
        rev = ${version};
      };

      nativeBuildInputs = [ cmake ];
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
    };
  };
}