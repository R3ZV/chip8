{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
    buildInputs = with pkgs; [
        cargo
        xorg.libX11
        xorg.libXi
        libGL
        libxkbcommon
    ];

    LD_LIBRARY_PATH = builtins.concatStringsSep ":" [
        "${pkgs.xorg.libX11}/lib"
        "${pkgs.xorg.libXi}/lib"
        "${pkgs.libGL}/lib"
        "${pkgs.libxkbcommon}/lib"
    ];
}
