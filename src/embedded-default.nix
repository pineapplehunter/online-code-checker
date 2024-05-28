{ pkgs ? import <nixpkgs> { } }:
let
  package = { stdenv }:
    stdenv.mkDerivation {
      name = "check";
      src = ./.;
      buildPhase = ''
        set -x
        if [ -f pattern.txt ]; do
          cat pattern.txt | while read line
          do
            grep $line main.c
          done
        done

        cc -o main main.c
        for i in $(cat pairs.txt); do
          diff -w <(cat input$i.txt | ./main) <(cat output$i.txt)
        done
        set +x
      '';
      installPhase = "touch $out";
    };
in
pkgs.callPackage package { }
