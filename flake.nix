{
  description = "A dev shell for the deltoid Rust package";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };
      in
      {
        formatter = pkgs.nixfmt-rfc-style;
        devShells = {
          default = pkgs.callPackage ./shell.nix { };
          buildInputs = [ ];
        };
      }
    );
}
