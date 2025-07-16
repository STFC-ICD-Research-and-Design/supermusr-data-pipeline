{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-viewer-tui = naersk'.buildPackage {
    name = "trace-viewer-tui";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-viewer-tui"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };

  trace-viewer-tui-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-trace-viewer-tui";
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
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${trace-viewer-tui}/bin/trace-viewer-tui"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      ];
    };
  };
}
