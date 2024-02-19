{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  nexus-writer = naersk'.buildPackage {
    name = "nexus-writer";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "nexus-writer"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  nexus-writer-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-nexus-writer";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${nexus-writer}/bin/nexus-writer"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}