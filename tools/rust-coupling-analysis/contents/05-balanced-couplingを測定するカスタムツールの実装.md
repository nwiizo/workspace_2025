## Balanced Couplingを測定するカスタムツールの実装

既存ツールは有用だが、Khononov のモデルを直接適用するには限界がある。特に、Integration Strength の分類、Dynamic Connascence の検出、Git 履歴との連携、Balance Score の計算といった機能が不足している。

これらを測定するため、カスタムツールを実装していこう。

### 基本的なアプローチ

必要な依存関係を `Cargo.toml` に追加します。

```toml
[dependencies]
syn = { version = "2.0", features = ["full", "visit"] }
quote = "1.0"
walkdir = "2.4"
thiserror = "2.0"
```

実装例です。

```rust
use syn::{visit::Visit, ItemFn, ItemImpl};

/// 結合度メトリクスを保持する構造体
///
/// Integration Strength、Distance、Connascenceの3つの次元を測定
#[derive(Debug, Default, Clone)]
pub struct CouplingMetrics {
    // Integration Strength
    pub intrusive_count: usize,
    pub functional_count: usize,
    pub model_count: usize,
    pub contract_count: usize,

    // Distance
    pub same_module: usize,
    pub cross_module: usize,
    pub cross_crate: usize,

    // Connascence
    pub name_coupling: Vec<String>,
    pub type_coupling: Vec<String>,
    pub position_coupling: Vec<String>,
}

/// ASTを訪問して結合度を分析するアナライザー
#[derive(Debug)]
pub struct CouplingAnalyzer {
    pub metrics: CouplingMetrics,
    pub current_module: String,
}

impl CouplingAnalyzer {
    /// 新しいアナライザーを作成
    ///
    /// # Arguments
    /// * `module_name` - 分析対象のモジュール名
    fn new(module_name: String) -> Self {
        Self {
            metrics: CouplingMetrics::default(),
            current_module: module_name,
        }
    }

    /// 関数シグネチャを分析してConnascence of Positionを検出
    ///
    /// # Arguments
    /// * `sig` - 分析対象の関数シグネチャ
    fn analyze_function_signature(&mut self, sig: &syn::Signature) {
        // 引数の数をチェック（Connascence of Position）
        if sig.inputs.len() > 3 {
            self.metrics.position_coupling.push(
                format!("Function {} has {} parameters",
                    sig.ident, sig.inputs.len())
            );
        }
    }

    /// 2つのモジュールパス間の距離を計算
    ///
    /// # Arguments
    /// * `from_path` - 開始パス (例: "crate::module::submodule")
    /// * `to_path` - 終了パス
    ///
    /// # Returns
    /// モジュール階層における段数（距離）
    fn calculate_distance(&self, from_path: &str, to_path: &str) -> usize {
        let from_parts: Vec<&str> = from_path.split("::").collect();
        let to_parts: Vec<&str> = to_path.split("::").collect();

        // 共通の祖先を見つける
        let common = from_parts.iter()
            .zip(to_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        (from_parts.len() - common) + (to_parts.len() - common)
    }
}

impl<'ast> Visit<'ast> for CouplingAnalyzer {
    /// 関数定義を訪問
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // 関数定義を分析
        self.analyze_function_signature(&node.sig);
        syn::visit::visit_item_fn(self, node);
    }

    /// implブロックを訪問してContract/Intrusive Couplingを検出
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        // トレイト実装を分析（Contract Coupling）
        if node.trait_.is_some() {
            self.metrics.contract_count += 1;
        } else {
            // 具象型への直接実装（Intrusive Coupling）
            self.metrics.intrusive_count += 1;
        }
        syn::visit::visit_item_impl(self, node);
    }
}
```

### Volatility（変動性）の測定

Git の履歴から変更頻度を分析します。

```rust
use std::process::Command;
use std::collections::HashMap;
use thiserror::Error;

/// Volatility分析のエラー型
#[derive(Error, Debug)]
pub enum VolatilityError {
    #[error("Git command failed: {0}")]
    GitCommandFailed(String),
    #[error("Failed to parse Git output: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// ファイルの変更頻度を分析するアナライザー
///
/// Git履歴を解析して各ファイルの変動性を評価
#[derive(Debug, Default, Clone)]
pub struct VolatilityAnalyzer {
    file_changes: HashMap<String, usize>,
}

/// 変動性のレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VolatilityLevel {
    /// 低変動性（汎用サブドメイン）
    Low,
    /// 中変動性（サポートサブドメイン）
    Medium,
    /// 高変動性（コアサブドメイン）
    High,
}

impl VolatilityAnalyzer {
    /// 新しいアナライザーを作成
    fn new() -> Self {
        Self::default()
    }

    /// Git履歴を解析して変更頻度を収集
    ///
    /// # Arguments
    /// * `months` - 過去何ヶ月分の履歴を分析するか
    ///
    /// # Returns
    /// 成功時は `Ok(())`、Git コマンド実行失敗時はエラー
    ///
    /// # Errors
    /// - Git コマンドが失敗した場合
    /// - 出力のパースに失敗した場合
    ///
    /// # Example
    /// ```
    /// let mut analyzer = VolatilityAnalyzer::new();
    /// analyzer.analyze_git_history(6)?; // 過去6ヶ月を分析
    /// ```
    pub fn analyze_git_history(&mut self, months: usize) -> Result<(), VolatilityError> {
        let output = Command::new("git")
            .args([
                "log",
                "--pretty=format:",
                "--name-only",
                &format!("--since={} months ago", months)
            ])
            .output()?;

        // Git コマンドが失敗した場合のチェック
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VolatilityError::GitCommandFailed(stderr.to_string()));
        }

        let files = String::from_utf8(output.stdout)?;
        for file in files.lines().filter(|l| !l.is_empty()) {
            *self.file_changes.entry(file.to_string())
                .or_insert(0) += 1;
        }

        Ok(())
    }

    /// ファイルの変動性レベルを分類
    ///
    /// # Arguments
    /// * `file` - 分類対象のファイルパス
    ///
    /// # Returns
    /// 変動性レベル（Low/Medium/High）
    fn classify_volatility(&self, file: &str) -> VolatilityLevel {
        let changes = self.file_changes.get(file).copied().unwrap_or(0);

        match changes {
            0..=2 => VolatilityLevel::Low,
            3..=10 => VolatilityLevel::Medium,
            _ => VolatilityLevel::High,
        }
    }

    /// 最も変更頻度の高いファイルを取得
    ///
    /// # Arguments
    /// * `n` - 取得する上位ファイル数
    ///
    /// # Returns
    /// (ファイルパス, 変更回数) のタプルのベクター
    fn top_volatile_files(&self, n: usize) -> Vec<(&str, usize)> {
        let mut files: Vec<_> = self.file_changes
            .iter()
            .map(|(file, &count)| (file.as_str(), count))
            .collect();

        files.sort_by(|a, b| b.1.cmp(&a.1));
        files.truncate(n);
        files
    }
}
```

### バランススコアの計算

```rust
/// Balancing Couplingのスコアを計算
///
/// Khononovのモデルに基づき、Strength、Distance、Volatilityの
/// 3次元からバランススコアを算出
#[derive(Debug, Clone)]
struct BalancedCouplingScore {
    /// 結合の強さ: 0.0 (弱い) から 1.0 (強い)
    strength: f64,
    /// 距離: 0.0 (近い) から 1.0 (遠い)
    distance: f64,
    /// 変動性: 0.0 (安定) から 1.0 (頻繁に変更)
    volatility: f64,
}

impl BalancedCouplingScore {
    /// 新しいスコアを作成
    ///
    /// # Arguments
    /// * `strength` - 結合の強さ (0.0-1.0)
    /// * `distance` - 距離 (0.0-1.0)
    /// * `volatility` - 変動性 (0.0-1.0)
    ///
    /// # Panics
    /// 各値が 0.0-1.0 の範囲外の場合にパニック
    fn new(strength: f64, distance: f64, volatility: f64) -> Self {
        assert!((0.0..=1.0).contains(&strength), "strength must be 0.0-1.0");
        assert!((0.0..=1.0).contains(&distance), "distance must be 0.0-1.0");
        assert!((0.0..=1.0).contains(&volatility), "volatility must be 0.0-1.0");

        Self {
            strength,
            distance,
            volatility,
        }
    }

    /// モジュラリティを計算
    ///
    /// # Formula
    /// MODULARITY = STRENGTH XOR DISTANCE (論理的な意味での排他的論理和)
    ///
    /// 理想的な状態：
    /// - 強い結合なら距離が近い (strength が高く distance が低い)
    /// - 弱い結合なら距離が遠くても良い (strength が低く distance が高い)
    ///
    /// # Returns
    /// モジュラリティスコア (0.0-1.0、高いほど良い)
    fn calculate_modularity(&self) -> f64 {
        // 強い結合 × 近い距離 = 良い（局所性が高い）
        let ideal_close = self.strength * (1.0 - self.distance);
        // 弱い結合 × 遠い距離 = 良い（疎結合が保たれている）
        let ideal_far = (1.0 - self.strength) * self.distance;

        ideal_close + ideal_far
    }

    /// バランススコアを計算
    ///
    /// # Formula
    /// BALANCE = MODULARITY OR (NOT VOLATILITY) (論理的な意味での論理和)
    ///
    /// バランスが取れている状態：
    /// - モジュラーである、または
    /// - 変動性が低い（安定している）
    ///
    /// # Returns
    /// バランススコア (0.0-1.0、高いほど良い)
    fn calculate_balance(&self) -> f64 {
        let modularity = self.calculate_modularity();
        let stability = 1.0 - self.volatility;

        // 論理和の近似：max を使用
        modularity.max(stability)
    }

    /// 結合の問題点を識別
    ///
    /// # Returns
    /// 検出された問題のリスト
    fn identify_issues(&self) -> Vec<CouplingIssue> {
        let mut issues = Vec::new();

        // パターン1: グローバル複雑性
        // 強い結合 + 遠い距離 = 変更の調整コストが高い
        if self.strength > 0.7 && self.distance > 0.7 {
            issues.push(CouplingIssue {
                severity: IssueSeverity::High,
                description: "Strong coupling over long distance increases global complexity"
                    .to_string(),
                recommendation: "Consider moving coupled components closer or reducing coupling strength"
                    .to_string(),
            });
        }

        // パターン2: ローカル複雑性
        // 弱い結合 + 近い距離 = 不要な抽象化の可能性
        if self.strength < 0.3 && self.distance < 0.3 {
            issues.push(CouplingIssue {
                severity: IssueSeverity::Medium,
                description: "Weak coupling at close distance may indicate unnecessary indirection"
                    .to_string(),
                recommendation: "Consider consolidating components or increasing distance"
                    .to_string(),
            });
        }

        // パターン3: カスケード変更リスク
        // 強い結合 + 高変動性 = 変更が広範囲に波及
        if self.strength > 0.7 && self.volatility > 0.7 {
            issues.push(CouplingIssue {
                severity: IssueSeverity::Critical,
                description: "Strong coupling with volatile component creates cascading change risk"
                    .to_string(),
                recommendation: "Isolate volatile components or reduce coupling strength"
                    .to_string(),
            });
        }

        // パターン4: 低モジュラリティ
        let modularity = self.calculate_modularity();
        if modularity < 0.4 {
            issues.push(CouplingIssue {
                severity: IssueSeverity::Medium,
                description: format!("Low modularity score: {:.2}", modularity),
                recommendation: "Review coupling strength and distance relationship"
                    .to_string(),
            });
        }

        issues
    }
}

/// 結合に関する問題
#[derive(Debug, Clone)]
struct CouplingIssue {
    severity: IssueSeverity,
    description: String,
    recommendation: String,
}

/// 問題の重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```
