{
  description = "Clause development and release environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/174eb786fb68e3a13e4e535a3deea479a0c07a6a";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/132a10336af9ae819bdf640c0dd1c789b12d7107";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rustToolchain = pkgs.rust-bin.stable."1.96.1".minimal.override {
        extensions = [ "rustfmt" "clippy" ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.stdenv.cc
        ];
      };
    };
}
