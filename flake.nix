# JOSSO development environment: tools to work with JOSSO/IAM.tf
{
  description = "VRH - PDF management module";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    devenv.url = "github:cachix/devenv";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    fenix,
  } @ inputs:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
          };
        };
      in
        with pkgs; {
          devShells.default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              ({
                pkgs,
                config,
                ...
              }: rec {
                languages.rust = {
                  enable = true;
                  # https://devenv.sh/reference/options/#languagesrustchannel
                  channel = "stable";
                  components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer"];
                };

                pre-commit.hooks = {
                  rustfmt.enable = true;
                  clippy.enable = true;
                };

                packages =
                  lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
                    frameworks.Security
                  ])
                  ++ (with pkgs; [ngrok nushell glibc just cargo-watch]);

                enterShell = with pkgs; ''
                  echo
                  echo "🦾 Atricore SOC - Auth0 log collector (rust)"
                  echo
                  echo ${builtins.concatStringsSep "," languages.rust.components}
                  echo $(rustc --version)
                  echo $(cargo --version)
                  echo
                  echo "Rust source path: "
                  echo "RUST_SRC_PATH=$RUST_SRC_PATH"
                  echo
                  echo other packages:
                  echo
                  echo ${cargo-watch.name}
                  echo ${glibc.name}
                  echo ${nushell.name}
                  echo ${just.name}
                  echo ${ngrok.name}
                '';
              })
            ];
          };
        }
    );
}
