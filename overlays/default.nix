final: prev: {
    tdengine = final.callPackage ./tdengine { nixpkgs = prev; };
    /* libuv = prev.libuv.overrideAttrs (oldAttrs: rec {
        version = "1.44.1";

        src = final.fetchFromGitHub {
            owner = oldAttrs.pname;
            repo = oldAttrs.pname;
            rev = "v${version}";
            sha256 = "sha256-12uveSEavRxQW4xVrB4Rkkj+eHZ71Qy8dRG+95ldz50=";
        };

        buildPhase = ''
            export LIBUV_BUILD_TESTS=false
        '';
    }); */
}