{
  description = "ccdirenv — direnv-style automatic Claude Code account switching";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in {
        packages = rec {
          ccdirenv = pkgs.rustPlatform.buildRustPackage {
            pname = "ccdirenv";
            version = cargoToml.package.version;

            src = pkgs.lib.cleanSource ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # ccdirenv has no native dependencies — pure Rust + libc.
            nativeBuildInputs = [ ];
            buildInputs = [ ];

            # Tests touch the filesystem (TempDir) and run subprocesses (cargo
            # for assert_cmd integration tests). They pass in CI containers and
            # locally, but the Nix sandbox can be stricter — keep them on but
            # skip a couple of integration tests that require a writable HOME.
            doCheck = true;

            meta = with pkgs.lib; {
              description = "direnv-style automatic Claude Code account switching";
              homepage = "https://github.com/SuguruOoki/ccdirenv";
              license = with licenses; [ mit asl20 ];
              maintainers = [ ];
              mainProgram = "ccdirenv";
              platforms = platforms.unix;
            };
          };
          default = ccdirenv;
        };

        apps = rec {
          ccdirenv = flake-utils.lib.mkApp { drv = self.packages.${system}.ccdirenv; };
          default = ccdirenv;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
          ];
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
