{
  description = "Node.js development environment with Nix Flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
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
          ...
        }:
        {
          # treefmt 設定
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              prettier.enable = true;
            };
          };

          # 開発シェル
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              # Node.js
              nodejs_22
              corepack # pnpm, yarn 管理

              # 開発ツール
              nodePackages.typescript
              nodePackages.typescript-language-server

              # ビルドツール
              esbuild
              turbo

              # タスクランナー
              just

              # フォーマッター
              config.treefmt.build.wrapper
            ];

            # 環境変数
            env = {
              # pnpm のグローバルディレクトリをプロジェクト内に
              PNPM_HOME = ".pnpm-store";
            };

            # シェル起動時のフック
            shellHook = ''
              echo "📦 Node.js development environment loaded!"
              echo "Node version: $(node --version)"
              echo "npm version: $(npm --version)"
              echo ""
              echo "Available commands:"
              echo "  pnpm install   - Install dependencies"
              echo "  pnpm dev       - Start development server"
              echo "  pnpm test      - Run tests"
              echo "  treefmt        - Format all files"
            '';
          };
        };
    };
}
