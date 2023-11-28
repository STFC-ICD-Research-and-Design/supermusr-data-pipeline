{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  kafka-daq-report = naersk'.buildPackage {
    name = "kafka-daq-report";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "kafka-daq-report"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  kafka-daq-report-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-kafka-daq-report";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${kafka-daq-report}/bin/kafka-daq-report"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
