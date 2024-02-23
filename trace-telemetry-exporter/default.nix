{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-telemetry-exporter = naersk'.buildPackage {
    name = "trace-telemetry-exporter";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-telemetry-exporter"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  trace-telemetry-exporter-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-telemetry-exporter";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-telemetry-exporter}/bin/trace-telemetry-exporter"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "METRICS_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
