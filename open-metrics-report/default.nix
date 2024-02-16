{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-telemetry-adapter = naersk'.buildPackage {
    name = "trace-telemetry-adapter";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-telemetry-adapter"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  trace-telemetry-adapter-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-telemetry-adapter";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-telemetry-adapter}/bin/trace-telemetry-adapter"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
