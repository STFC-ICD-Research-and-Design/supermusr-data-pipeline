{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
  hdf5-joined,
}: rec {
  stream-to-file = naersk'.buildPackage {
    name = "stream-to-file";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "stream-to-file"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };

    HDF5_DIR = "${hdf5-joined}";
  };

  stream-to-file-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-stream-to-file";
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
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${stream-to-file}/bin/stream-to-file"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
