{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.rust-analyzer
          ];

          buildInputs = [ 
            (rustVersion.override { extensions = [ "rust-src" ]; })
            rustToolchain
            pkgs.pkg-config
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.xorg.libxcb
            pkgs.wayland-protocols
            pkgs.just
            pkgs.vulkan-loader
            pkgs.mesa
            pkgs.vulkan-tools
          ];

          shellHook = ''
            echo "🦀 Rust + Wayland dev shell"
            export RUST_SRC_PATH=${rustToolchain}/lib/rustlib/src/rust/library
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader pkgs.mesa ]}:$LD_LIBRARY_PATH"
          '';
        };
      });
}
