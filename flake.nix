{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat = {
      url = "github:edolstra/flake-compat/master";
      flake = false;
    };
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      eachSystem =
        f:
        nixpkgs.lib.genAttrs
          [
            "aarch64-darwin"
            "aarch64-linux"
            "x86_64-darwin"
            "x86_64-linux"
          ]
          (
            system:
            f (
              import nixpkgs {
                inherit system;
                config.allowUnfree = true;
                overlays = [];
              }
            )
          );
    in
    {
      formatter = eachSystem (pkgs: pkgs.nixfmt-rfc-style);
      packages = eachSystem (pkgs: {
        default = pkgs.callPackage (
          { lib, rustPlatform }:
          let
            manifest = lib.importTOML ./Cargo.toml;
          in
          rustPlatform.buildRustPackage {
            pname = manifest.package.name;
            inherit (manifest.package) version;
            src = lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
          }
        ) { };
      });
      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.system}.default ];
          packages = with pkgs; [
            cargo
            cargo-expand
            cargo-watch
            clippy
            evcxr
            mold
            rust-analyzer
            rustc
            rustfmt
            sccache
          ];
          CARGO_INCREMENTAL = "0";
          RUSTC_WRAPPER = "sccache";
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });
    };
}
