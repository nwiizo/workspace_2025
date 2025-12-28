{
  description = "Rust development environment with Nix Flake";

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
          # rust-overlay を適用した pkgs
          overlayPkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          # 安定版 Rust ツールチェーン
          rustToolchain = overlayPkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
            ];
          };

          # Nightly Rust（最新機能が必要な場合）
          # rustToolchainNightly = overlayPkgs.rust-bin.nightly.latest.default;
        in
        {
          # treefmt 設定
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
            };
          };

          # 開発シェル
          devShells.default = pkgs.mkShell {
            packages = [
              # Rust ツールチェーン
              rustToolchain
              pkgs.cargo-watch
              pkgs.cargo-edit
              pkgs.cargo-audit

              # ビルドツール
              pkgs.pkg-config
              pkgs.openssl

              # フォーマッター
              config.treefmt.build.wrapper

              # 追加ツール
              pkgs.just
              pkgs.tokei
            ];

            # 環境変数
            env = {
              RUST_BACKTRACE = "1";
              RUST_LOG = "debug";
            };

            # シェル起動時のフック
            shellHook = ''
              echo "🦀 Rust development environment loaded!"
              echo "Rust version: $(rustc --version)"
              echo "Cargo version: $(cargo --version)"
              echo ""
              echo "Available commands:"
              echo "  cargo build    - Build the project"
              echo "  cargo test     - Run tests"
              echo "  cargo watch    - Watch for changes"
              echo "  treefmt        - Format all files"
            '';
          };
        };
    };
}
