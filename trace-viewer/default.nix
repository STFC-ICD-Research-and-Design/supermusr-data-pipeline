{
  pkgs,
  naersk',
  version,
  gitRevision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  trace-viewer = naersk'.buildPackage {
    name = "trace-viewer";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "trace-viewer"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

    overrideMain = p: {
      GIT_REVISION = gitRevision;
    };
  };
}
