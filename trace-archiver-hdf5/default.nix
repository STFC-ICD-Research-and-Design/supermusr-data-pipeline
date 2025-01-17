{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-archiver-hdf5 = naersk'.buildPackage {
    name = "trace-archiver-hdf5";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-archiver-hdf5"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };

  trace-archiver-hdf5-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-archiver-hdf5";
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
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-archiver-hdf5}/bin/trace-archiver-hdf5"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
