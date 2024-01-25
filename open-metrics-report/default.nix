{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  open-metrics-report = naersk'.buildPackage {
    name = "open-metrics-report";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "open-metrics-report"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  open-metrics-report-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-open-metrics-report";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${open-metrics-report}/bin/open-metrics-report"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
