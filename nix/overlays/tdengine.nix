self: super: {
  tdengine =
  let
    version = "3.0.4.2";
  in super.gcc9Stdenv.mkDerivation {
    name = "TDengine-client";
    version = version;

    src = super.fetchFromGitHub  {
      owner = "taosdata";
      repo = "TDEngine";
      rev = "ver-${version}";
      hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
    };
  /*
    src = super.fetchgit  {
      url = "https://github.com/taosdata/TDengine";
      #owner = "taosdata";
      #repo = "TDEngine";
      rev = "ver-${version}";
      hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
      fetchSubmodules = true;
    };
*/
    dontUseCmakeConfigure=false;
    
    SSL_CERT_FILE = "${super.cacert}/etc/ssl/certs/ca-bundle.crt";
    outputHash = "sha256-pQpattmS9VmO3ZIQUFn66az8GSmB4IvYhTTCFn6SUmo=";
    outputHashMode = "recursive";

    nativeBuildInputs = with super; [
      cmake
      cacert
      git
      libuv
    ];

    #buildPhase = ''
    #  bash ./build.sh
    #'';
    # The "-DBUILD...=false" options come from https://github.com/taosdata/TDengine/blob/main/cmake/cmake.options
    configPhase = ''
    '';

    #buildPhase = ''
    #  cmake .
    #  make -j
    #'';
    
    #installPhase = ''
    #  make install
    #'';
  };
}