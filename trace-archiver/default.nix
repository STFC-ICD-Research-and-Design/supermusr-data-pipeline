{
  pkgs,
  naersk',
  version,
  git_revision,
  buildInputs,
  nativeBuildInputs,
} : rec {
  package = naersk'.buildPackage {
    name = "trace-archiver";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-archiver"];

    nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  container-image = pkgs.dockerTools.buildImage {
    name = "trace-archiver";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [ bashInteractive coreutils ];
      pathsToLink = [ "/bin" ];
    };

    config = {
      ExposedPorts = {
        "9090/tcp" = {};
      };
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}