{
  description = "Go development environment with Nix Flake";

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
              gofmt.enable = true;
            };
          };

          # 開発シェル
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              # Go 本体
              go

              # 開発ツール
              golangci-lint
              gotools
              gopls
              delve
              gotestsum

              # ビルド・タスクランナー
              just

              # フォーマッター
              config.treefmt.build.wrapper
            ];

            # 環境変数
            env = {
              CGO_ENABLED = "0";
              GOFLAGS = "-mod=readonly";
            };

            # シェル起動時のフック
            shellHook = ''
              echo "🐹 Go development environment loaded!"
              echo "Go version: $(go version)"
              echo ""
              echo "Available commands:"
              echo "  go build       - Build the project"
              echo "  go test ./...  - Run tests"
              echo "  golangci-lint  - Run linter"
              echo "  air            - Hot reload development"
              echo "  treefmt        - Format all files"
            '';
          };
        };
    };
}
