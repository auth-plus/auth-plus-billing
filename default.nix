{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage {
  pname = "auth-plus-billing";
  version = "0.1.0";

  nativeBuildInputs = with pkgs; [ pkg-config cmake ];
  buildInputs = with pkgs; [ openssl ];

  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
}
