# Neovimで使うCopilotのモデルをClaudeに変更する苦労話 - 技術ブログ未満の個人的体験談

![Neovim+Copilot+Claude](/api/placeholder/800/400 "Neovim Copilot Claude")

> **免責事項**: この記事は個人的な発見と試行錯誤を記録したものであり、正式なドキュメントに基づく推奨設定ではありません。ここで紹介する方法を実際の環境に適用する際は、十分な検証と自己責任でお願いします。

こんにちは、Neovimユーザーのnwiizoです。今回は、NeovimでCopilotを使う際にAIモデルをClaudeに変更しようとして遭遇した「ちょっとした冒険」について共有したいと思います。

## やりたかったこと

最近、GitHub Copilotが Claude-3.7-sonnetなどのAnthropicのモデルをサポートするようになり、コーディング支援にもっと高度なAIの力を借りたいと思いました。GitHubの公式ドキュメント「[Copilot Chat の AI モデルを変更する](https://docs.github.com/ja/copilot/using-github-copilot/ai-models/changing-the-ai-model-for-copilot-chat)」によると、Copilot ChatのデフォルトLLMを別のモデルに変更できるようになっています。

私のNeovim環境では`yetone/avante.nvim`を使用してCopilotとの対話を行っていたので、このプラグインの設定でモデルを変更しようとしました。

## 最初の試み（失敗）

まず試したのは、`avante.nvim`の設定で直接モデルを指定する方法です：

```lua
{
    "yetone/avante.nvim",
    event = "VeryLazy",
    lazy = false,
    version = false,
    opts = {
      provider = "copilot", -- copilotを使用
      auto_suggestions_provider = "copilot",
      copilot = {
        endpoint = "https://api.githubcopilot.com",
        model = "claude-3.7-sonnet", -- ここでClaudeモデルを指定
        timeout = 30000,
        temperature = 0,
        max_tokens = 4096,
      },
      -- 以下省略...
    },
    -- 依存関係などの設定...
}
```

しかし、この設定を適用してもなぜか期待通りの動作をしませんでした。デバッグを試みましたが、`avante.nvim`の設定だけではモデルの変更がうまく反映されていないようでした。

## 解決策：CopilotChat.nvimの力を借りる

調査を進めるうちに、[CopilotC-Nvim/CopilotChat.nvim](https://github.com/CopilotC-Nvim/CopilotChat.nvim)というプラグインがあることを知りました。このプラグインはCopilotのチャットインターフェースを提供するもので、モデル設定も直接サポートしています。

試しに以下の設定を追加してみました：

```lua
-- Copilotチャット用の設定
{
  "CopilotC-Nvim/CopilotChat.nvim",
  event = { "VeryLazy" },
  branch = "main",
  dependencies = {
    { "zbirenbaum/copilot.lua" },
    { "nvim-lua/plenary.nvim" },
  },
  opts = {
    model = "claude-3.7-sonnet", -- モデル名を指定
    debug = true, -- デバッグを有効化
  },
}
```

そして驚いたことに、この設定を追加した後、`avante.nvim`側でもClaudeモデルが使われるようになりました！どうやら、`CopilotChat.nvim`の設定が`copilot.lua`の基本設定に影響を与え、それが`avante.nvim`にも「引きずられて」反映されたようです。

**【注意】これは技術ブログ未満の個人的な発見であり、正式なドキュメントに基づくものではありません。このような依存関係による予期せぬ影響は、本番環境では深刻な問題を引き起こす可能性があります。設定の影響範囲を十分理解せずに適用すれば、システム全体に致命的な影響を及ぼす可能性があることを肝に銘じてください。**

## なぜこうなったのか？

正確な理由は不明ですが、おそらく両方のプラグインが同じ[zbirenbaum/copilot.lua](https://github.com/zbirenbaum/copilot.lua)に依存しており、この共通の依存関係を通じて設定が共有されたのだと思われます。

`CopilotChat.nvim`がより直接的にCopilot APIとの連携部分を制御しているため、そちらでの設定が優先されたのでしょう。

## 教訓

[Neovim](https://neovim.io/)のプラグインエコシステムでは、依存関係の連鎖によって予想外の相互作用が発生することがあります。今回のケースでは幸いにも望んだ結果につながりましたが、これは完全に「技術ブログ未満」の個人的な発見に過ぎません。

**警告**: このような依存関係の連鎖による予期せぬ相互作用は、本番環境では極めて危険です。プラグイン間の隠れた依存関係による設定の「引きずり」は、デバッグが困難な問題を引き起こし、最悪の場合、本番システムの停止や重大なセキュリティ問題につながる可能性があります。設定変更の影響範囲を完全に理解しないまま適用することは、言わば地雷原を歩くようなものであることを忘れないでください。

他の方も同様の状況に遭遇した場合、両方のプラグインを併用する方法が一つの解決策になるかもしれません。

## 最終的な設定

結局、私の設定は`avante.nvim`と`CopilotChat.nvim`の両方を含む形になりました：

```lua
-- avante.nvimの設定（一部省略）
{
    "yetone/avante.nvim",
    -- 省略...
    opts = {
      provider = "copilot",
      auto_suggestions_provider = "copilot",
      copilot = {
        endpoint = "https://api.githubcopilot.com",
        model = "claude-3.7-sonnet",
        -- 省略...
      },
    },
    -- 省略...
},

-- CopilotChat.nvimの設定
{
  "CopilotC-Nvim/CopilotChat.nvim",
  event = { "VeryLazy" },
  branch = "main",
  dependencies = {
    { "zbirenbaum/copilot.lua" },
    { "nvim-lua/plenary.nvim" },
  },
  opts = {
    model = "claude-3.7-sonnet",
    debug = true,
  },
},
```

これでNeovimでのコーディング体験がClaudeの能力で強化され、より的確なコード提案や説明が得られるようになりました。

Neovimの設定は時に「魔法」のように思えることもありますが、それも含めて楽しいハック体験の一部なのでしょう。

---

*注：この記事は2025年2月時点の情報に基づいています。Copilotの仕様やプラグインの動作は変更される可能性があります。*

## 参考リンク

- [GitHub Copilot Chat の AI モデルを変更する方法](https://docs.github.com/ja/copilot/using-github-copilot/ai-models/changing-the-ai-model-for-copilot-chat)
- [yetone/avante.nvim](https://github.com/yetone/avante.nvim)
- [CopilotC-Nvim/CopilotChat.nvim](https://github.com/CopilotC-Nvim/CopilotChat.nvim)
- [zbirenbaum/copilot.lua](https://github.com/zbirenbaum/copilot.lua)
