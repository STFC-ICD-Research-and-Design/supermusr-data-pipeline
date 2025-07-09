{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-server = naersk'.buildPackage {
    name = "trace-server";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-server"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };

  trace-server-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-server";
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
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-server}/bin/trace-server"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
