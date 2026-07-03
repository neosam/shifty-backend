{
  description = "Shifty Backend - Shiftyplan Service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    openspec.url = "github:Fission-AI/OpenSpec";
    gsd.url = "github:neosam/gsd-flake";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, openspec, gsd }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.minimal.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Frontend build (Dioxus/WASM). The Cargo manifest at
        # shifty-dioxus/Cargo.toml uses `path = "../rest-types"`, so the
        # build src must include both shifty-dioxus/ and rest-types/. We
        # take the whole repo and `cd shifty-dioxus` in the build phase.
        frontend-build = pkgs.rustPlatform.buildRustPackage {
          pname = "shifty-dioxus";
          version = "2.2.2-dev";

          src = ./.;
          setSourceRoot = ''
            sourceRoot=$(echo */shifty-dioxus | head -n1)
          '';

          cargoLock = {
            lockFile = ./shifty-dioxus/Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            rustToolchain
            wasm-pack
            wasm-bindgen-cli_0_2_121
            nodejs # provides npm
            tailwindcss
            pkg-config
            dioxus-cli
            binaryen
            hexdump
            removeReferencesTo
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

          buildPhase = ''
            runHook preBuild

            export HOME=$TMPDIR
            export CARGO_HOME=$TMPDIR/.cargo
            export DIOXUS_WASM_OPT_DISABLE=1

            echo "Building Tailwind CSS..."
            if [ -f "./input.css" ]; then
              tailwindcss -i ./input.css -o ./assets/tailwind.css --minify
            fi

            echo "Building Dioxus frontend..."
            mkdir -p dist
            cargo build --target wasm32-unknown-unknown --release

            if [ -f "target/wasm32-unknown-unknown/release/shifty-dioxus.wasm" ]; then
              cp target/wasm32-unknown-unknown/release/shifty-dioxus.wasm dist/

              wasm-bindgen --out-dir dist --target web target/wasm32-unknown-unknown/release/shifty-dioxus.wasm

              echo "Stripping debug symbols and optimizing WASM..."
              find dist -name "*.wasm" -type f | while read wasm_file; do
                echo "Optimizing $wasm_file with wasm-opt..."
                ls -lh "$wasm_file"
                ${pkgs.hexdump}/bin/hexdump -C "$wasm_file" | head -n 10 || echo "(failed to hexdump)"
                wasm-opt \
                  -Oz \
                  --enable-bulk-memory \
                  --enable-mutable-globals \
                  --strip-debug \
                  --strip-dwarf \
                  --strip-producers \
                  "$wasm_file" -o "$wasm_file.tmp"
                mv "$wasm_file.tmp" "$wasm_file"
                remove-references-to -t ${rustToolchain} "$wasm_file"
              done

              cat > dist/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Shifty</title>
    <link rel="stylesheet" href="/tailwind.css">
</head>
<body>
    <div id="main"></div>
    <script type="module">
        import init from '/./shifty-dioxus.js';
        init();
    </script>
</body>
</html>
EOF

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
              find $out -type f \( -name "*.wasm" -o -name "*.js" \) -exec \
                remove-references-to -t ${rustToolchain} {} \;
            else
              echo "Warning: dist directory not found"
              mkdir -p $out
              echo "Build failed - no dist output" > $out/error.txt
            fi

            runHook postInstall
          '';

          dontCargoCheck = true;
          dontCargoBuild = true;

          meta = with pkgs.lib; {
            description = "Shifty Frontend - Dioxus/WASM";
            license = licenses.mit;
            platforms = platforms.all;
          };
        };
      in
      {
        packages = {
          # Backend mit mock_auth (default)
          default = pkgs.callPackage ./default.nix {
            inherit pkgs;
            features = [ "mock_auth" ];
          };

          # Backend mit mock_auth
          backend-mock = pkgs.callPackage ./default.nix {
            inherit pkgs;
            features = [ "mock_auth" "" ];
          };

          # Backend mit OIDC
          backend-oidc = pkgs.callPackage ./default.nix {
            inherit pkgs;
            features = [ "oidc" "json_logging" ];
          };

          # Frontend (Dioxus/WASM, baut shifty-dioxus + rest-types co-located)
          frontend = pkgs.runCommand "shifty-dioxus" {
            allowReferences = [ ];
          } ''
            mkdir -p $out
            cp -r ${frontend-build}/* $out/
          '';
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            cargo-watch
            cargo-audit
            clippy
            rust-analyzer
            sqlx-cli
            sqlite
            nodejs
            pkg-config
            # lld is required for linking the frontend's wasm32 build gate
            # (cargo build --target wasm32-unknown-unknown). The default linker
            # cannot link the wasm target.
            lld

            openspec.packages.${system}.default
            gsd.packages.${system}.default
          ];

          # Target-specific: only affects the wasm32 frontend build gate, the
          # native backend build keeps its default linker.
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
      }
    ) // {
      # NixOS Module (system-unabhängig)
      nixosModules.default = import ./module.nix;
    };
}
