{
  description = "A thorough example of a featureful REST API in Axum";

  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix";
    cargo2nix.flake = false;
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    flake-utils.url = "github:numtide/flake-utils";
    master.url = "nixpkgs/master";
    nixpkgs.url = "nixpkgs/nixos-21.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "unstable";
    unstable.url = "nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ self, flake-utils, nixpkgs, ... }:
    flake-utils.lib.eachSystem [ "x86_64-darwin" "x86_64-linux" ] (system:
      let
        master = import inputs.master { inherit system; };
        unstable = import inputs.unstable { inherit system; };
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
            (import inputs.rust-overlay)
            (import "${inputs.cargo2nix}/overlay")
            (final: prev: { inherit master unstable; })
            (final: prev:
              let
                cargo2nixPkgs = import inputs.cargo2nix {
                  inherit (inputs) nixpkgs rust-overlay;
                  inherit system;
                  rustChannel = "stable";
                };
              in { cargo2nix = cargo2nixPkgs.package.bin; })
          ];
        };

        sharedInputs = with pkgs;
          [
            cargo-asm
            cargo-audit
            cargo-bloat
            cargo-cache
            cargo-deny
            cargo-edit
            cargo-expand
            cargo-flamegraph
            cargo-generate
            cargo-geiger
            cargo-make
            cargo-outdated
            cargo-release
            cargo-sweep
            cargo-udeps
            cargo-watch
            cargo-web
            cargo-whatfeatures
            clang
            just
            lld
            mdbook
            openssl.dev
            pkg-config
            # Out-of-order intentional for PATH priority
            self.outputs.packages.${system}.rust-analyzer
            self.outputs.packages.${system}.sqlx-cli
            # rustEnv
            sccache
            zlib.dev
          ] ++ pkgs.lib.optionals (pkgs.stdenv.isDarwin)
          (with pkgs.darwin.apple_sdk.frameworks; [
            Security
            SystemConfiguration
          ]) ++ lib.optionals (stdenv.isLinux) [
            cargo-tarpaulin
            perf-tools
            strace
            valgrind
          ];
      in {
        devShell = pkgs.mkShell {
          buildInputs = sharedInputs
            ++ [ (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain) ];

          NIX_PATH = "nixpkgs=${nixpkgs}:unstable=${inputs.unstable}";
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          # RUSTC_WRAPPER = "${pkgs.unstable.sccache}/bin/sccache";
        };

        packages = let
          # https://github.com/cargo2nix/cargo2nix/issues/187#issuecomment-828181090
          inputIconv = name:
            pkgs.rustBuilder.rustLib.makeOverride {
              inherit name;
              overrideAttrs = drv: {
                propagatedBuildInputs = drv.propagatedBuildInputs or [ ]
                  ++ [ pkgs.libiconv ]
                  ++ pkgs.lib.optional (pkgs.stdenv.isDarwin)
                  pkgs.darwin.apple_sdk.frameworks.Security;
              };
            };
          rustPkgs = pkgs.rustBuilder.makePackageSet' {
            rustChannel =
              pkgs.lib.removeSuffix "\n" (builtins.readFile ./rust-toolchain);
            packageFun = import ./Cargo.nix;
            localPatterns = [
              "^(bin|src|tests)(/.*)?"
              "[^/]*\\.(rs|toml)$"
              "sqlx-data\\.json$"
            ];
            packageOverrides = pkgs:
              pkgs.rustBuilder.overrides.all ++ (builtins.map inputIconv [
                "libc"
                "log"
                "memchr"
                "axum_rest_example"
                "proc-macro2"
              ]) ++ [
                (pkgs.rustBuilder.rustLib.makeOverride {
                  name = "axum_rest_example";
                  overrideAttrs = { SQLX_OFFLINE = "true"; };
                })
              ];
          };
        in {
          gcroot = pkgs.linkFarmFromDrvs "axum_rest_example"
            (with self.outputs; [ devShell."${system}".inputDerivation ]);

          cargo2nix = pkgs.cargo2nix;
          rust-analyzer = pkgs.master.rust-analyzer;
          sqlx-cli = pkgs.sqlx-cli;

          axum_rest_example = (rustPkgs.workspace.axum_rest_example { }).bin;

          inherit (rustPkgs) workspace;

          nightlyDevShell = pkgs.mkShell {
            buildInputs = sharedInputs
              ++ [ pkgs.rust-bin.nightly.latest.default ];
          };

          docker = pkgs.dockerTools.streamLayeredImage {
            name = "axum_rest_example";
            tag = "latest";
            contents =
              [ self.outputs.packages.x86_64-linux.axum_rest_example.bin ];
            config = {
              Cmd =
                [ self.outputs.packages.x86_64-linux.axum_rest_example.bin ];
              Env =
                [ "RUST_LOG=axum_rest_example=trace,tower_http=trace,info" ];
            };
          };
        };

        defaultPackage = self.outputs.packages.${system}.axum_rest_example;
      });
}
