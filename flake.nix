{
  description = "FFI-compatible and no-std &CStr";
  inputs = {
    flakelib.url = "github:flakelib/fl";
    nixpkgs = { };
    rust = {
      url = "github:arcnmx/nixexprs-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, flakelib, nixpkgs, rust, ... }@inputs: let
    nixlib = nixpkgs.lib;
  in flakelib {
    inherit inputs;
    systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    devShells = {
      plain = {
        mkShell, writeShellScriptBin, hostPlatform
      , pkg-config
      , glib, libiconv
      , enableRustdoc ? false
      , enableRust ? true, cargo
      , rustTools ? [ ]
      }: mkShell {
        inherit rustTools;
        buildInputs = [ glib ]
          ++ nixlib.optionals hostPlatform.isDarwin [ libiconv ];
        nativeBuildInputs = [
          pkg-config
        ] ++ nixlib.optional enableRust cargo;
        RUSTDOCFLAGS = rust.lib.rustdocFlags {
          inherit (self.lib) crate;
          enableUnstableRustdoc = enableRustdoc;
        };
      };
      stable = { rust'stable, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (rust'stable) mkShell;
        enableRust = false;
      };
      dev = { rust'unstable, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (rust'unstable) mkShell;
        enableRust = false;
        enableRustdoc = true;
        rustTools = [ "rust-analyzer" ];
      };
      default = { outputs'devShells }: outputs'devShells.plain;
    };
    checks = {
      rustfmt = { rust'builders, source }: rust'builders.check-rustfmt-unstable {
        src = source;
        config = ./.rustfmt.toml;
      };
      version = { rust'builders, source }: rust'builders.check-contents {
        src = source;
        patterns = [
          { path = "src/lib.rs"; docs'rs = {
            inherit (self.lib.crate.package) name version;
          }; }
        ];
      };
      test = { outputs'devShells'plain, rustPlatform, source, features ? [ ] }: rustPlatform.buildRustPackage {
        pname = self.lib.crate.package.name;
        inherit (self.lib.crate.package) version;
        inherit (outputs'devShells'plain.override { enableRust = false; }) buildInputs nativeBuildInputs;
        cargoLock.lockFile = ./Cargo.lock;
        buildNoDefaultFeatures = true;
        buildFeatures = features;
        src = source;
        buildType = "debug";
        meta.name = "cargo test";
      };
    };
    legacyPackages = { callPackageSet }: callPackageSet {
      source = { rust'builders }: rust'builders.wrapSource self.lib.crate.src;
    } { };
    lib = with nixlib; {
      crate = rust.lib.importCargo ./Cargo.toml;
      inherit (self.lib.crate.package) version;
    };
    config = rec {
      name = "cstrptr";
      packages.namespace = [ name ];
    };
  };
}
