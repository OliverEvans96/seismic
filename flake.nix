{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, fenix, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustPlatform = pkgs.rustPlatform;
      in rec {
        defaultPackage = rustPlatform.buildRustPackage {
          pname = "seismic";
          version = "0.1.0";

          nativeBuildInputs = with pkgs; [ lld pkgconfig udev ];

          cargoLock = { lockFile = ./Cargo.lock; };

          src = ./.;
        };

        packages = {
          dockerImage = pkgs.dockerTools.buildImage {
            name = "seismic-docker";
            tag = "latest";
            # Config options reference:
            # https://github.com/moby/moby/blob/master/image/spec/v1.2.md#image-json-field-descriptions
            config.Cmd = [ "${defaultPackage}/bin/server" ];
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              pathsToLink = [ "/bin" ];
              paths = with pkgs; [
                bash # bash
                coreutils # ls, cat, etc
                inetutils # ip, ifconfig, etc.
                iana-etc # /etc/protocols
                netcat-gnu # nc
                defaultPackage # this application
              ];
            };
          };
        };

        devShell = pkgs.mkShell {
          name = "seismic-shell";
          src = ./.;

          # build-time deps
          nativeBuildInputs =
            (with pkgs; [ rustc cargo openssl lld pkgconfig udev ]);
        };
      });
}
