## はじめに

2025年、Rustのコンテナ化は劇的な進化を遂げました。cargo-chefとBuildKitキャッシュマウントの組み合わせで**ビルド時間を5-10倍短縮**し、distrolessイメージで**約50MB**、musl静的リンクなら**わずか1.7MB**という驚異的な小ささを実現しています。

本記事では、実践的なDockerfileパターンとベンチマーク結果を詳細に解説します。**実際に検証したAxum Webアプリケーション**では、distroless版で50.3MB、musl+scratch版で1.71MBを達成しました。中規模プロジェクト（~500依存関係）では、初回ビルド10分です。コード変更後の再ビルドはわずか40秒です。

[https://github.com/LukeMathWalker/cargo-chef:embed:cite]

