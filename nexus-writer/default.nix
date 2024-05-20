{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
  hdf5-joined,
}: rec {
  nexus-writer = naersk'.buildPackage {
    name = "nexus-writer";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "nexus-writer"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };

    HDF5_DIR = "${hdf5-joined}";
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
      ExposedPorts = {
        "9090/tcp" = {};
      };
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${nexus-writer}/bin/nexus-writer"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
