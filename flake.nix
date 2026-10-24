{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:

    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              config.allowUnfree = true;
              config.permittedInsecurePackages = [
                "python3.14-ecdsa-0.19.2"
              ];
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine (
            (with latest; [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
              llvm-tools-preview
            ])
            ++ [
              targets.thumbv7em-none-eabi.latest.rust-std
            ]
          );
      };

      devShells = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              cargo-binutils
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
              probe-rs-tools
              self.formatter.${system}
              llvmPackages.libclang
              python3
              python3.pkgs.pip
              python3.pkgs.cryptography
              python3.pkgs.intelhex
              python3.pkgs.click
              python3.pkgs.cbor2
              python3.pkgs.setuptools
              python3.pkgs.pyyaml
              python3.pkgs.pytest
              adafruit-nrfutil
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                pkgs.llvmPackages.libclang
              ];
            };
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt-rs);
    };
}
