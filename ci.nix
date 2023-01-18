{ pkgs, channels, lib, config, ... }: with pkgs; with lib; let
  inherit (import ./. { inherit pkgs; }) checks inputs;
in {
  config = {
    name = "cstrptr";
    ci.gh-actions.enable = true;
    cache.cachix = {
      ci.signingKey = "";
      arc.enable = true;
    };
    channels = {
      nixpkgs = "22.11";
    };
    tasks = {
      version.inputs = singleton checks.version;
      build.inputs = singleton (checks.test.override {
        inherit (config.cstrptr) rustPlatform features;
      });
    };
    jobs = let
      rustPlatforms = {
        stable = pkgs.rustPlatform;
        nightly = inputs.rust.legacyPackages.${system}.unstable.rustPlatform;
      };
      channels = [ "stable" "nightly" ];
      features = [ "std" "alloc" "unstable" "memchr" ];
      validate = channel: features:
        (channel == "nightly" || all (f: f != "unstable") features) && # unstable feature can only be used on nightly
        count (f: f == "std" || f == "alloc") features < 2; # combining std with alloc is meaningless
      features' = genList (_: features ++ [ null ]) (length features + 1);
      filtered = list: unique (filter (f: f != null) list);
      permuted = unique (crossLists (a: b: c: d: e: filtered [ a b c d e ]) features');
    in listToAttrs (filter (v: v != null) (flip crossLists [ channels permuted ] (channel: features: if validate channel features
      then nameValuePair "${channel}${optionalString (features != []) "-${concatStringsSep "-" features}"}" ({ ... }: {
        cstrptr = {
          rustPlatform = rustPlatforms.${channel};
          inherit features;
        };
      }) else null
    )));
  };
  options.cstrptr = with types; {
    rustPlatform = mkOption {
      type = unspecified;
      default = pkgs.rustPlatform;
    };
    features = mkOption {
      type = listOf str;
      default = [ "default" ];
    };
  };
}
