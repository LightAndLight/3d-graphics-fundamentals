{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    ipso.url = "github:lightandlight/ipso";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ipso }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };

        rustVersion = "1.70.0";
      
      in {
        devShell =
          pkgs.mkShell {
            buildInputs = with pkgs; [
              ipso.defaultPackage.${system}
              (rust-bin.stable.${rustVersion}.default.override {
                extensions = [
                  "cargo"
                  "clippy"
                  "rustc"
                  "rust-src"
                  "rustfmt"
                ];
              })
              
              renderdoc
            ];
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              vulkan-loader
            ];
          };
      }
    );
}
