{
  description = "Rust OS Kernel Development Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }: 
  let
    system = "x86_64-linux";
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };

    rust = pkgs.rust-bin.nightly."2024-11-22".default.override {
      targets = [ "x86_64-unknown-none" ];
      extensions = [ "rust-src" "rustc-dev" "llvm-tools-preview" ];
    };

  in {
    devShells.${system}.default = pkgs.mkShell rec {
      nativeBuildInputs = [
        pkgs.pkg-config
      ];
      buildInputs = [
        pkgs.wayland
        pkgs.vulkan-loader
        pkgs.wayland
        pkgs.wayland-protocols
        pkgs.xorg.libXcursor
        pkgs.xorg.libXrandr
        pkgs.xorg.libXi
        pkgs.vulkan-loader
        pkgs.libxkbcommon
        rust
        pkgs.SDL2
        pkgs.rust-analyzer
        pkgs.clippy
      ];

      shellHook = ''
        export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}
        exec zsh -c "nvim"
      '';
    };
  };
}

