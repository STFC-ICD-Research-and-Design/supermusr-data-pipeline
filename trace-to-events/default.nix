{
  pkgs,
  naersk',
  version,
  git_revision,
  buildInputs,
  nativeBuildInputs,
} : rec {
  package = naersk'.buildPackage {
    name = "trace-to-events";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-to-events"];

    nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];
    buildInputs = buildInputs;

    # Ensure ffmpeg binary is available
    postInstall = ''
      wrapProgram $out/bin/trace-to-events --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.ffmpeg ]}
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
        ${package}/bin/trace-to-events "$@"
      '';
    };
  in
  pkgs.dockerTools.buildImage {
    name = "trace-to-events";
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