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
      in {
        defaultPackage = rustPlatform.buildRustPackage {
          pname = "seismic";
          version = "0.1.0";

          nativeBuildInputs = with pkgs; [ lld pkgconfig udev ];

          cargoLock = { lockFile = ./Cargo.lock; };

          src = ./.;
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
