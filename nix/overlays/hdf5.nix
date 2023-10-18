self: super: {
  hdf5 = super.hdf5.overrideAttrs (old: rec {
    version = "1.12.2";
    src = super.fetchurl {
      url = "https://support.hdfgroup.org/ftp/HDF5/releases/hdf5-${super.lib.versions.majorMinor version}/hdf5-${version}/src/hdf5-${version}.tar.bz2";
      sha256 = "sha256-Goi742ITos6gyDlyAaRZZD5xVcnckeBiZ1s/sH7jiv4=";
    };
  });
}
