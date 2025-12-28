{
  description = "Multi-language development environment with Nix Flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];

      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        let
          # Rust overlay
          overlayPkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          rustToolchain = overlayPkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          # Python 環境
          pythonEnv = pkgs.python312.withPackages (
            ps: with ps; [
              pip
              pytest
              mypy
            ]
          );

          # 共通ツール
          commonTools = with pkgs; [
            just
            jq
            yq-go
            config.treefmt.build.wrapper
          ];
        in
        {
          # treefmt 設定（全言語対応）
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              gofmt.enable = true;
              ruff.enable = true;
              prettier.enable = true;
            };
          };

          # デフォルト: 全言語入り
          devShells.default = pkgs.mkShell {
            packages = commonTools ++ [
              # Rust
              rustToolchain
              pkgs.cargo-watch

              # Go
              pkgs.go
              pkgs.golangci-lint
              pkgs.gopls

              # Python
              pythonEnv
              pkgs.uv
              pkgs.ruff

              # Node.js
              pkgs.nodejs_22
              pkgs.corepack
            ];

            shellHook = ''
              echo "🌐 Multi-language development environment loaded!"
              echo ""
              echo "Loaded languages:"
              echo "  🦀 Rust: $(rustc --version)"
              echo "  🐹 Go:   $(go version)"
              echo "  🐍 Python: $(python --version)"
              echo "  📦 Node.js: $(node --version)"
              echo ""
              echo "Use 'nix develop .#rust' for Rust-only environment"
              echo "Use 'nix develop .#go' for Go-only environment"
              echo "Use 'nix develop .#python' for Python-only environment"
              echo "Use 'nix develop .#nodejs' for Node.js-only environment"
            '';
          };

          # Rust のみ
          devShells.rust = pkgs.mkShell {
            packages = commonTools ++ [
              rustToolchain
              pkgs.cargo-watch
              pkgs.cargo-edit
            ];
            shellHook = ''
              echo "🦀 Rust development environment loaded!"
            '';
          };

          # Go のみ
          devShells.go = pkgs.mkShell {
            packages = commonTools ++ [
              pkgs.go
              pkgs.golangci-lint
              pkgs.gopls
              pkgs.delve
            ];
            shellHook = ''
              echo "🐹 Go development environment loaded!"
            '';
          };

          # Python のみ
          devShells.python = pkgs.mkShell {
            packages = commonTools ++ [
              pythonEnv
              pkgs.uv
              pkgs.ruff
              pkgs.pyright
            ];
            shellHook = ''
              echo "🐍 Python development environment loaded!"
            '';
          };

          # Node.js のみ
          devShells.nodejs = pkgs.mkShell {
            packages = commonTools ++ [
              pkgs.nodejs_22
              pkgs.corepack
              pkgs.nodePackages.typescript
            ];
            shellHook = ''
              echo "📦 Node.js development environment loaded!"
            '';
          };
        };
    };
}
