{
  description = "Docker integration with Nix Flake - Build reproducible container images";

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
          lib,
          ...
        }:
        let
          # Rust overlay
          overlayPkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          rustToolchain = overlayPkgs.rust-bin.stable.latest.minimal;

          # サンプルアプリケーション（Hello World）
          helloApp = pkgs.writeShellScriptBin "hello-app" ''
            echo "Hello from Nix-built Docker image!"
            echo "Container built at: $(date)"
          '';
        in
        {
          packages = {
            # シンプルな Docker イメージ
            docker-simple = pkgs.dockerTools.buildImage {
              name = "nix-hello";
              tag = "latest";

              # コンテンツ（パッケージ）
              copyToRoot = pkgs.buildEnv {
                name = "image-root";
                paths = [
                  helloApp
                  pkgs.coreutils
                  pkgs.bash
                ];
                pathsToLink = [ "/bin" ];
              };

              config = {
                Cmd = [ "/bin/hello-app" ];
                WorkingDir = "/";
              };
            };

            # レイヤード Docker イメージ（効率的なキャッシュ）
            docker-layered = pkgs.dockerTools.buildLayeredImage {
              name = "nix-app-layered";
              tag = "latest";

              # 各依存がレイヤーになる
              contents = [
                helloApp
                pkgs.coreutils
                pkgs.bash
                pkgs.cacert  # SSL証明書
              ];

              config = {
                Cmd = [ "/bin/hello-app" ];
                Env = [
                  "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                ];
              };

              # レイヤー数の制限
              maxLayers = 100;
            };

            # 最小限の Distroless スタイルイメージ
            docker-minimal = pkgs.dockerTools.buildImage {
              name = "nix-minimal";
              tag = "latest";

              # 必要最小限のファイルのみ
              copyToRoot = pkgs.buildEnv {
                name = "minimal-root";
                paths = [ helloApp ];
                pathsToLink = [ "/bin" ];
              };

              config = {
                Cmd = [ "/bin/hello-app" ];
                # ユーザー設定（セキュリティ）
                User = "1000:1000";
              };
            };

            # ストリーム方式（メモリ効率が良い）
            docker-stream = pkgs.dockerTools.streamLayeredImage {
              name = "nix-stream";
              tag = "latest";

              contents = [
                helloApp
                pkgs.bash
              ];

              config = {
                Cmd = [ "/bin/hello-app" ];
              };
            };

            default = helloApp;
          };

          # 開発シェル（Docker CLI も含む）
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              docker
              dive  # Docker イメージ分析ツール
              skopeo  # コンテナイメージ操作
              just
            ];

            shellHook = ''
              echo "🐳 Docker + Nix development environment"
              echo ""
              echo "Available commands:"
              echo "  nix build .#docker-simple  - Build simple Docker image"
              echo "  nix build .#docker-layered - Build layered Docker image"
              echo "  nix build .#docker-minimal - Build minimal Docker image"
              echo ""
              echo "After building, load the image:"
              echo "  docker load < result"
              echo ""
              echo "Or stream directly:"
              echo "  nix build .#docker-stream --json | jq -r '.[0].outputs.out' | xargs -I{} bash {} | docker load"
            '';
          };
        };
    };
}
