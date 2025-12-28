{
  description = "Python development environment with Nix Flake";

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
        let
          # 開発用 Python 環境
          pythonEnv = pkgs.python312.withPackages (
            ps: with ps; [
              # パッケージ管理
              pip
              setuptools
              wheel

              # 開発ツール
              pytest
              pytest-cov
              pytest-asyncio
              mypy
              ipython
            ]
          );
        in
        {
          # treefmt 設定
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              ruff.enable = true;
            };
          };

          # 開発シェル
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              # Python 環境
              pythonEnv
              uv # 高速パッケージマネージャ

              # Lint & Format
              ruff
              pyright

              # 追加ツール
              just
              config.treefmt.build.wrapper
            ];

            # 環境変数
            env = {
              PYTHONDONTWRITEBYTECODE = "1";
              PYTHONUNBUFFERED = "1";
              # 仮想環境をプロジェクト内に作成
              UV_PROJECT_ENVIRONMENT = ".venv";
            };

            # シェル起動時のフック
            shellHook = ''
              echo "🐍 Python development environment loaded!"
              echo "Python version: $(python --version)"
              echo "uv version: $(uv --version)"
              echo ""
              echo "Available commands:"
              echo "  uv sync        - Install dependencies"
              echo "  uv run pytest  - Run tests"
              echo "  ruff check .   - Run linter"
              echo "  treefmt        - Format all files"
            '';
          };
        };
    };
}
