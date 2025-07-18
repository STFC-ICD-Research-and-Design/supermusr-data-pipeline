{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-reader = naersk'.buildPackage {
    name = "trace-reader";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-reader"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };

  trace-reader-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-reader";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-reader}/bin/trace-reader"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
