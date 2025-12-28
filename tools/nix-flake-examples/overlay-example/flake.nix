{
  description = "Nix Flake with custom overlay examples";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        let
          # カスタムオーバーレイを定義
          customOverlay = final: prev: {
            # 1. 既存パッケージをラップ
            # git に追加のデフォルトオプションを設定
            myGit = prev.writeShellScriptBin "git" ''
              exec ${prev.git}/bin/git -c init.defaultBranch=main "$@"
            '';

            # 2. パッケージのバージョンを固定
            # 特定バージョンの Node.js を使用
            nodejs-fixed = prev.nodejs_20;

            # 3. シンプルなカスタムスクリプト
            project-init = prev.writeShellScriptBin "project-init" ''
              echo "🚀 Initializing new project..."
              ${prev.git}/bin/git init
              echo "# New Project" > README.md
              echo "*.log" > .gitignore
              echo "✅ Project initialized!"
            '';

            # 4. 複数ツールを組み合わせたスクリプト
            dev-check = prev.writeShellScriptBin "dev-check" ''
              echo "🔍 Running development checks..."
              echo ""
              echo "📦 Git status:"
              ${prev.git}/bin/git status --short 2>/dev/null || echo "  Not a git repository"
              echo ""
              echo "🔧 Environment:"
              echo "  PATH entries: $(echo $PATH | tr ':' '\n' | wc -l)"
              echo "  Nix store paths: $(echo $PATH | tr ':' '\n' | grep -c '/nix/store' || echo 0)"
              echo ""
              echo "✅ Check complete!"
            '';
          };

          # オーバーレイを適用した pkgs
          customPkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ customOverlay ];
          };
        in
        {
          # カスタムパッケージをエクスポート
          packages = {
            default = customPkgs.project-init;
            project-init = customPkgs.project-init;
            dev-check = customPkgs.dev-check;
          };

          # 開発シェル
          devShells.default = pkgs.mkShell {
            packages = [
              # オーバーレイで定義したパッケージ
              customPkgs.project-init
              customPkgs.dev-check
              customPkgs.myGit

              # 標準パッケージ
              pkgs.just
              pkgs.jq
            ];

            shellHook = ''
              echo "🎨 Custom overlay environment loaded!"
              echo ""
              echo "Custom commands available:"
              echo "  project-init  - Initialize a new project"
              echo "  dev-check     - Run development checks"
              echo "  git           - Git with custom defaults"
              echo ""
              echo "Try 'nix run .#project-init' to run directly"
            '';
          };
        };
    };
}
