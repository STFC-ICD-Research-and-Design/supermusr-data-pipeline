self: super: {
  tdengine =
  let
    stdenv = super.stdenv;
    version = "3.0.4.2";
  in super.stdenv.mkDerivation {
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
    dontUseCmakeConfigure=true;
    
    SSL_CERT_FILE = "${super.cacert}/etc/ssl/certs/ca-bundle.crt";
    outputHash = "sha256-I4UGDcrtmX/1TAQz89peXsqoetZmCM+1b3XYqexv/VA=";
    outputHashMode = "recursive";

    nativeBuildInputs = with super; [
      cmake
      cacert
      git
      libuv
      #zlib
      pkg-config
      #xz
      #jansson
      #apr
      #aprutil
      #curl
    ];

    #buildPhase = ''
    #  bash ./build.sh
    #'';
    # The "-DBUILD...=false" options come from https://github.com/taosdata/TDengine/blob/main/cmake/cmake.options
    buildPhase = ''
      cmake -DBUILD_WITH_UV=false -DBUILD_WITH_ROCKSDB=false -DBUILD_WITH_LEVELDB=false -DBUILD_ADDR2LINE=false -DBUILD_WITH_UV_TRANS=false -DBUILD_DOCS=false
      make -j
    '';
    
    installPhase = ''
    '';
  };
}