{ pkgs ? import <nixpkgs> { } }: with pkgs; mkShell {
  packages = [
    musl
  ];
}
