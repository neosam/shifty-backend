{ pkgs ? import <nixpkgs> }:
let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustPlatform = pkgs.rust-bin.stable."1.90.0";
in pkgs.rustPlatform.buildRustPackage rec {
  pname = "shifty-dioxus";
  version = "1.9.0-dev";
  
  src = ./.;
  
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    # Rust with WebAssembly support
    (rustPlatform.default.override {
      extensions = [ "rust-src" ];
      targets = [ "wasm32-unknown-unknown" ];
    })
    
    # Required for WebAssembly builds
    wasm-pack
    wasm-bindgen-cli
    
    # Dioxus CLI from nixpkgs
    dioxus-cli
    
    # Node.js for Tailwind CSS
    nodejs
    nodePackages.npm
    tailwindcss
    
    # System dependencies
    pkg-config
  ];

  buildInputs = with pkgs; [
    openssl
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # Environment for WebAssembly builds
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  
  # Custom build phase for Dioxus
  buildPhase = ''
    runHook preBuild
    
    export HOME=$TMPDIR
    export CARGO_HOME=$TMPDIR/.cargo
    
    # Disable wasm-opt to avoid parsing errors
    export DIOXUS_WASM_OPT_DISABLE=1
    
    echo "Building Tailwind CSS..."
    if [ -f "./input.css" ]; then
      tailwindcss -i ./input.css -o ./assets/tailwind.css --minify
    fi
    
    echo "Building Dioxus frontend..."
    # Try building with cargo directly to avoid wasm-opt issues
    mkdir -p dist
    cargo build --target wasm32-unknown-unknown --release
    
    # Copy the wasm file and create basic HTML
    if [ -f "target/wasm32-unknown-unknown/release/shifty-dioxus.wasm" ]; then
      cp target/wasm32-unknown-unknown/release/shifty-dioxus dist/
      
      # Generate JS bindings with wasm-bindgen
      wasm-bindgen --out-dir dist --target web target/wasm32-unknown-unknown/release/shifty-dioxus.wasm
      
      # Create index.html
      cat > dist/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Inventurly</title>
    <link rel="stylesheet" href="tailwind.css">
</head>
<body>
    <div id="main"></div>
    <script type="module">
        import init from './shifty-dioxus.js';
        init();
    </script>
</body>
</html>
EOF
      
      # Copy CSS if it exists
      if [ -f "assets/tailwind.css" ]; then
        cp assets/tailwind.css dist/
      fi
    else
      echo "Error: WASM file not found"
      exit 1
    fi
    
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    
    mkdir -p $out
    if [ -d "dist" ]; then
      cp -r dist/* $out/
    else
      echo "Warning: dist directory not found"
      mkdir -p $out
      echo "Build failed - no dist output" > $out/error.txt
    fi
    
    runHook postInstall
  '';

  # Don't run standard cargo build
  dontCargoCheck = true;
  dontCargoBuild = true;

  meta = with pkgs.lib; {
    description = "Inventurly Frontend - Inventory Management System built with Dioxus";
    license = licenses.mit;
    platforms = platforms.all;
  };
}
