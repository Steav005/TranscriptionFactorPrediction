{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, devshell, fenix, ... }@inputs:
    utils.lib.eachSystem [ "aarch64-linux" "i686-linux" "x86_64-linux" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ devshell.overlay ];
          };
          rust-toolchain = with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              stable.clippy
              stable.llvm-tools-preview
              latest.rustfmt
              # rust-analyzer
            ];
          my-python-packages = python-packages: [
            python-packages.pip
            python-packages.wheel
            python-packages.pytest
            # python-packages.python-lsp-server
            # python-packages.pylsp-mypy
            # python-packages.mypy
          ];
          my-python = pkgs.python310.withPackages my-python-packages;
        in
        rec {
          devShells.default = (pkgs.devshell.mkShell {
            imports = [ "${devshell}/extra/git/hooks.nix" ];
            name = "rpy-dev-shell";
            packages = with pkgs; [
              clang
              rust-toolchain
              cargo-outdated
              cargo-udeps
              cargo-audit
              cargo-expand
              cargo-all-features
              cargo-watch
              cargo-llvm-cov
              nixpkgs-fmt
              rust-analyzer

              maturin
              my-python
            ];
            git.hooks = {
              enable = true;
              pre-commit.text = ''
                nix flake check
              '';
            };
            env = [
              # {
              #   name = "RUSTFLAGS";
              #   value = " -C instrument-coverage --cfg coverage --cfg trybuild_no_target";
              # }
              # {
              #   name = "LLVM_PROFILE_FILE";
              #   value = "$PRJ_ROOT/rust-python-coverage/target/rust-python-coverage-%m.profraw";
              # }
              # {
              #   name = "CARGO_INCREMENTAL";
              #   value = "0";
              # }
              # {
              #   name = "CARGO_LLVM_COV_TARGET_DIR";
              #   value = "$PRJ_ROOT/rust-python-coverage/target";
              # }
              {
                name = "PIP_PREFIX";
                value = "$PRJ_ROOT/_build/pip_packages";
              }
              {
                name = "PYTHONPATH";
                prefix =
                  "$PRJ_ROOT/_build/pip_packages/lib/python3.9/site-packages";
              }
              {
                name = "SOURCE_DATE_EPOCH";
                unset = true;
              }

            ];
            commands = [
              { package = "git-cliff"; }
              { package = "treefmt"; }
              {
                name = "udeps";
                command = ''
                  PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                  cargo udeps $@
                '';
                help = pkgs.cargo-udeps.meta.description;
              }
              {
                name = "outdated";
                command = "cargo outdated $@";
                help = pkgs.cargo-outdated.meta.description;
              }
              {
                name = "audit";
                command = "cargo audit $@";
                help = pkgs.cargo-audit.meta.description;
              }
              {
                name = "expand";
                command = ''
                  PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                  cargo expand $@
                '';
                help = pkgs.cargo-expand.meta.description;
              }
            ];
          });
          checks = {
            nixpkgs-fmt = pkgs.runCommand "nixpkgs-fmt"
              {
                nativeBuildInputs = [ pkgs.nixpkgs-fmt ];
              } "nixpkgs-fmt --check ${./.}; touch $out";
            cargo-fmt = pkgs.runCommand "cargo-fmt"
              {
                nativeBuildInputs = [ rust-toolchain ];
              } "cd ${./.}; cargo fmt --check; touch $out";
          };
        });
}

