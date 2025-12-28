# mosaic-video

MP3音声ファイルを入力として、ランダムなモザイクパターンの映像と共に動画ファイル（MP4）を出力するRust製CLIツール。

## 必要条件

- Rust 2024 edition
- FFmpeg（システムにインストールされている必要があります）

### FFmpegのインストール

**macOS:**
```bash
brew install ffmpeg
```

**Ubuntu/Debian:**
```bash
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**Windows:**
FFmpegをダウンロードしてPATHに追加してください。

## インストール

```bash
cargo install --path .
```

または、ビルドのみ：

```bash
cargo build --release
```

## 使用方法

### 基本的な使用法

```bash
mosaic-video input.mp3 -o output.mp4
```

### フルオプション

```bash
mosaic-video input.mp3 \
  --output output.mp4 \
  --width 1280 \
  --height 720 \
  --fps 24 \
  --block-size 32 \
  --change-interval 0.5
```

## オプション一覧

| オプション | 短縮形 | デフォルト | 説明 |
|-----------|-------|-----------|------|
| `--output` | `-o` | `output.mp4` | 出力ファイル名 |
| `--width` | `-W` | `1280` | 動画の幅（px） |
| `--height` | `-H` | `720` | 動画の高さ（px） |
| `--fps` | `-f` | `24` | フレームレート |
| `--block-size` | `-b` | `32` | モザイクブロックサイズ（px） |
| `--change-interval` | `-i` | `0.5` | モザイク変更間隔（秒） |

## 出力例

```
mosaic-video - MP3 to Video Converter
======================================

Input:  music.mp3
Output: output.mp4

Analyzing audio file...
  Duration:    180.50 seconds
  Sample rate: 44100 Hz
  Channels:    2

Video settings:
  Resolution:  1280x720
  FPS:         24
  Block size:  32px
  Change interval: 0.50s

Generating video...
⠋ [00:00:15] [########################################] 4332/4332 frames (0s)
Merging audio with video...
Video created successfully: output.mp4

Done! Video saved to: output.mp4
```

## ライセンス

MIT
