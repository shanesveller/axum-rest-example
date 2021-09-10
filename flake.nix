{
  description = "A thorough example of a featureful REST API in Axum";

  inputs = {
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
            (final: prev: { inherit master unstable; })
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

        packages = {
          gcroot = pkgs.linkFarmFromDrvs "axum_rest_example"
            (with self.outputs; [ devShell."${system}".inputDerivation ]);

          rust-analyzer = pkgs.master.rust-analyzer;
          sqlx-cli = pkgs.sqlx-cli;

          nightlyDevShell = pkgs.mkShell {
            buildInputs = sharedInputs
              ++ [ pkgs.rust-bin.nightly.latest.default ];
          };
        };
      });
}
