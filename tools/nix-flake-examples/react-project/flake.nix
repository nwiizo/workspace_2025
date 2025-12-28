{
  description = "React development environment with Nix Flake";

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
              corepack # pnpm, yarn

              # TypeScript
              nodePackages.typescript
              nodePackages.typescript-language-server

              # ビルド・バンドル
              esbuild

              # Lint & Format
              nodePackages.eslint
              nodePackages.prettier

              # 追加ツール
              just
              jq
              config.treefmt.build.wrapper
            ];

            # 環境変数
            env = {
              # Node.js メモリ上限
              NODE_OPTIONS = "--max-old-space-size=4096";
            };

            # シェル起動時のフック
            shellHook = ''
              echo "⚛️  React development environment loaded!"
              echo ""
              echo "Node.js: $(node --version)"
              echo "npm: $(npm --version)"
              echo ""
              echo "Quick start:"
              echo "  npm create vite@latest my-app -- --template react-ts"
              echo "  cd my-app && npm install && npm run dev"
              echo ""
              echo "Or with pnpm:"
              echo "  corepack enable pnpm"
              echo "  pnpm create vite my-app -- --template react-ts"
              echo ""
              echo "Available commands:"
              echo "  npm run dev     - Start Vite dev server"
              echo "  npm run build   - Build for production"
              echo "  npm run preview - Preview production build"
              echo "  treefmt         - Format all files"
            '';
          };

          # Vite + React テンプレート作成用
          devShells.init = pkgs.mkShell {
            packages = with pkgs; [
              nodejs_22
              corepack
            ];
            shellHook = ''
              echo "Creating new React + Vite project..."
              npm create vite@latest . -- --template react-ts
              echo "Done! Run 'nix develop' to enter the development environment."
            '';
          };
        };
    };
}
