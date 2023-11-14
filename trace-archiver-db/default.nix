{ pkgs ? import <nixpkgs> {} }:
pkgs.callPackage derivation.nix {}

{
  pkgs,
  naersk',
  version,
  git_revision,
  buildInputs,
  nativeBuildInputs,
} : rec {
  package = naersk'.buildPackage {
    name = "trace-archiver-db";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-archiver-db"];

    nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  container-image = pkgs.dockerTools.buildImage {
    name = "trace-archiver-db";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [ bashInteractive coreutils ];
      pathsToLink = [ "/bin" ];
    };

    config = {
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}