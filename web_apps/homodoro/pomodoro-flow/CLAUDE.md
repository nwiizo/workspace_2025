# Pomodoro Flow - AI開発ガイドライン

## プロジェクト概要
ポモドーロ・テクニック実践サイト - 集中力を最大化するタイムマネジメントツール

## 技術スタック
- **フロントエンド**: Next.js 14 (App Router) + TypeScript
- **バックエンド**: Cloudflare Workers + Hono
- **データベース**: Cloudflare D1 + Drizzle ORM
- **UI**: shadcn/ui + Tailwind CSS
- **状態管理**: React Query (TanStack Query)
- **API**: OpenAPI + Orval (自動生成)
- **モノレポ**: Turborepo + pnpm

## プロジェクト構成
```
pomodoro-flow/
├── apps/
│   ├── web/          # Next.js フロントエンド
│   └── api/          # Hono APIサーバー (Cloudflare Workers)
├── packages/
│   ├── ui/           # 共通UIコンポーネント
│   ├── database/     # Drizzleスキーマ & Prisma定義
│   ├── api-client/   # Orval生成のAPIクライアント
│   └── types/        # 共通型定義
└── CLAUDE.md         # このファイル
```

## コーディング規約

### TypeScript
- strict: true を必須
- 型推論を活用し、明示的な型定義は必要最小限に
- anyの使用禁止

### React/Next.js
- Server Componentsを優先
- Client Componentsは必要最小限
- use clientは必要な場合のみ

### API設計
- RESTful原則に従う
- OpenAPIスキーマから型を自動生成
- エラーハンドリングは一貫性を保つ

### データベース
- Prismaでスキーマ定義
- DrizzleでCloudflare D1操作
- マイグレーションは慎重に

## 開発フロー
1. Prismaスキーマを更新
2. Drizzleスキーマを自動生成
3. APIエンドポイント実装
4. OpenAPIスキーマ生成
5. Orvalでクライアントコード生成
6. フロントエンド実装

## 重要な原則
- **型安全性**: DBからUIまで一貫した型定義
- **パフォーマンス**: バンドルサイズと初回ロードを最小化
- **保守性**: 自動生成を活用し、手動コードを減らす
- **コスト効率**: Cloudflareの無料枠を最大限活用

## コマンド
```bash
# 開発
pnpm dev

# ビルド
pnpm build

# 型チェック
pnpm typecheck

# リント
pnpm lint

# データベースマイグレーション
pnpm db:migrate

# API型生成
pnpm generate:api
```

## 注意事項
- コメントは最小限に（コードで意図を表現）
- 絵文字の使用は避ける
- テストは必須（unit/integration/e2e）
- コミット前に必ずlint/typecheck実行