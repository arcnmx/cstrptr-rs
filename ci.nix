{ pkgs, channels, lib, config, ... }: with pkgs; with lib; let
  src = nix-gitignore.gitignoreSourcePure [ ''
    *.nix
    /.git
    /.github
  '' ./.gitignore ] ./.;
  cmd = name: command: ci.commandCC {
    inherit name;
    inherit src;
    #nativeBuildInputs = [ config.cstrptr.rustPlatform.cargo ];
    impure = true;
    environment = [ "CARGO_TARGET_DIR" ];
    command = ''
      ${config.cstrptr.rustPlatform.cargo}/bin/${command} --features "${concatStringsSep "," config.cstrptr.features}" --no-default-features
    '';
  };
in {
  name = "cstrptr";
  ci.gh-actions.enable = true;
  cache.cachix.arc.enable = true;
  channels = {
    nixpkgs = "21.05";
    rust = "master";
  };
  tasks.test.inputs = {
    test = cmd "test" "cargo test --manifest-path $src/Cargo.toml";
    build = cmd "build" "cargo build --manifest-path $src/Cargo.toml";
    doc = cmd "doc" "cargo doc --manifest-path $src/Cargo.toml";
  };
  imports = [ { options.cstrptr = {
    rustPlatform = mkOption {
      type = types.unspecified;
      default = channels.rust.stable.rustPlatform;
    };
    features = mkOption {
      type = types.listOf types.str;
      default = [ "default" ];
    };
  }; } ];
  jobs = let
    channels = [ "stable" "beta" "nightly" ];
    features = [ "std" "alloc" "unstable" "memchr" ];
    validate = channel: features:
      (channel == "nightly" || all (f: f != "unstable") features) && # unstable feature can only be used on nightly
      count (f: f == "std" || f == "alloc") features < 2; # combining std with alloc is meaningless
    features' = genList (_: features ++ [ null ]) (length features + 1);
    filtered = list: unique (filter (f: f != null) list);
    permuted = unique (crossLists (a: b: c: d: e: filtered [ a b c d e ]) features');
  in listToAttrs (filter (v: v != null) (flip crossLists [ channels permuted ] (channel: features: if validate channel features
    then nameValuePair "${channel}${optionalString (features != []) "-${concatStringsSep "-" features}"}" ({ channels, ... }: {
      cstrptr = {
        rustPlatform = channels.rust.${channel}.rustPlatform;
        inherit features;
      };
    }) else null
  ))) // {
    shell = { channels, ... }: {
      ci.gh-actions.emit = false;
      channels.arc = "master";
      environment.shell.rust = channels.arc.pkgs.rustPlatforms.nightly.mkShell {
        cargoCommands = [ "clippy" ];
        rustTools = [ "rust-analyzer" ];
      };
    };
  };
}
