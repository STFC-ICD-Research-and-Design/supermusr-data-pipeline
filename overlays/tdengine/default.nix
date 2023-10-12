{
  nixpkgs,
  stdenv,
  fetchurl
}:
  let
    version = "3.0.4.2";
  in stdenv.mkDerivation {
  name = "TDengine-client";
  version = version;

  src = fetchurl {
    url = "https://www.taosdata.com/assets-download/3.0/TDengine-client-${version}-Linux-x64.tar.gz";
    hash = "sha256-7qshbjOKF9fHpaT7UNAUlQAMtWh1BN/GSwKe2/k3VF0=";
  };


  nativeBuildInputs = with nixpkgs; [
    sudo
  ];

  installPhase = ''
    sudo bash ./install_client.sh
  '';
}