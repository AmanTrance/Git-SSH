{ pkgs ? import <nixpkgs> { } } :
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    zlib
    glibc
    libz
  ];

  LD_LIBRARY_PATH = "${pkgs.zlib.outPath}/lib:${pkgs.glibc.outPath}/lib:${pkgs.libz.outPath}/lib";
}
