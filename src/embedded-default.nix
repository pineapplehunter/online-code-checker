{pkgs?import <nixpkgs> {}}: with pkgs; stdenv.mkDerivation {
  name="check";
  src = ./.;
  buildPhase = ''
    set -x
    cc -o main main.c
    for i in $(cat pairs.txt); do
      diff -w <(cat input$i.txt | ./main) <(cat output$i.txt)
    done
    set +x
  '';
  installPhase = "touch $out";
}