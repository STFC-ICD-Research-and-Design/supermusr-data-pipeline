{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: {
  trace-to-events = naersk'.buildPackage {
    name = "trace-to-events";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-to-events"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  trace-to-events-container-image = pkgs.dockerTools.buildImage {
    name = "trace-to-events";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      ExposedPorts = {
        "9090/tcp" = {};
      };
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-to-events}/bin/trace-to-events"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
