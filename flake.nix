{
  description = "Rust with WebAssembly";

  inputs = {
    # Add the unstable channel
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable"; 
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          # Import the unstable packages
          pkgs-unstable = import nixpkgs-unstable { inherit system; };

          wasm-rust-toolchain = with fenix.packages.${system}; combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
        in
          {
            devShells.default =
              pkgs.mkShell {
                name = "rust-wasm-final-attempt";
                packages = [
                  wasm-rust-toolchain
                  pkgs.llvmPackages.bintools 
                  pkgs.dioxus-cli 
                  pkgs.wasm-bindgen-cli
                  pkgs.wasm-pack
                  
                  # Use the unstable version of binaryen (contains updated wasm-opt)
                  pkgs-unstable.binaryen 
                ];
                
                CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
              };
          }
      );
}
