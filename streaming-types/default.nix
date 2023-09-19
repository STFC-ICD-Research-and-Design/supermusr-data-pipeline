{
  pkgs,
  naersk',
  version,
  git_revision,
  buildInputs,
  nativeBuildInputs,
} : rec {
  package = naersk'.buildPackage {
    name = "streaming-types";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "streaming-types"];

    nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];
    buildInputs = buildInputs;

    # Ensure ffmpeg binary is available
    postInstall = ''
      wrapProgram $out/bin/streaming-types --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.ffmpeg ]}
    '';
    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  container-image = let
    entrypoint = pkgs.writeShellApplication {
      name = "entrypoint";
      text = ''
        #!${pkgs.runtimeShell}
        mkdir -m 1777 /tmp
        ${package}/bin/streaming-types "$@"
      '';
    };
  in
  pkgs.dockerTools.buildImage {
    name = "streaming-types";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [ bashInteractive coreutils ];
      pathsToLink = [ "/bin" ];
    };

    config = {
      Entrypoint = [ "${pkgs.tini}/bin/tini" "--" "${entrypoint}/bin/entrypoint" ];
      ExposedPorts = {
        "8000/tcp" = {};
        "9090/tcp" = {};
      };
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "HTTP_SERVER_ADDRESS=0.0.0.0:8000"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}