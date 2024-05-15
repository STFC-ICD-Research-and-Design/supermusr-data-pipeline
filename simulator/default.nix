{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  simulator = naersk'.buildPackage {
    name = "simulator";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "simulator"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };

  simulator-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-simulator";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${simulator}/bin/simulator"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
