{
  description = "A simple Rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    (flake-utils.lib.eachDefaultSystem 
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          rustPackage = pkgs.rustPlatform.buildRustPackage rec {
            pname = "discord-nlp-bot";
            src = self;
            cargoSha256 = "sha256-8UBiY6JwgCj8Q1BC0CF+oO8rGo9W/xOgutoPTAkGrmY=";
          };
        in
        {
          packages.default = rustPackage;
          defaultPackage = rustPackage;
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              fontconfig
              pkg-config
            ];
            nativeBuildInputs = with pkgs; [
              fontconfig
              pkg-config
            ];
          };
        }
      )
    );
}
