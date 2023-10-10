{ pkgs }: {
  default = final: _prev: {
    tdengine-client = pkgs.callPackage ./tdengine {pkgs = final;};
  };
}