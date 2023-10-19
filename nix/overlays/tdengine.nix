self: super: {
  tdengine =
  let
    stdenv = super.stdenv;
    version = "3.0.4.2";
  in super.stdenv.mkDerivation {
    name = "TDengine-client";
    version = version;

    src = super.fetchGitHub  {
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
      zlib
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
    ];

    #buildPhase = ''
    #  bash ./build.sh
    #'';
    buildPhase = ''
      mkdir debug
      cd debug
      cmake .. -DBUILD_WITH_UV=true
      ls build
      make -j client
      ls
      cd client
    '';
    
    installPhase = ''
    '';
  };
}