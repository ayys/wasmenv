{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
        {
          defaultPackage = naersk-lib.buildPackage ./.;
          packages = rec {
            wasmenv = naersk-lib.buildPackage ./.;
          };
          devShell = with pkgs; mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              openssl.dev # Add OpenSSL development libraries
            ];

            # Environment variables for `openssl-sys` crate
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            RUSTFLAGS = "-C link-arg=-L${pkgs.openssl.out}/lib";            
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
        }
    );
}
