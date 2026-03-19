# WayLog CLI

[![GitHub license](https://img.shields.io/github/license/shayne-snap/waylog-cli?style=flat-square)](https://github.com/shayne-snap/waylog-cli/blob/main/LICENSE)
![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?style=flat-square)

**Seamlessly sync, preserve, and version-control your AI coding conversations locally.**

WayLog CLI is a lightweight tool written in Rust that automatically saves your AI coding sessions (Claude Code, Gemini CLI, OpenAI Codex CLI) into clean, searchable local Markdown files. Stop losing your context to session timeouts—WayLog CLI helps you own your AI history locally.

[中文文档](README_zh.md) | [English](README.md)

---

## ✨ Features

- **🔄 Auto-Sync**: Real-time synchronization of chat history to `~/.waylog/history/<project_name>` as you type.
- **📦 Full History Recovery**: The `pull` command scans your entire machine to recover past sessions into the current project.
- **📝 Markdown Native**: All history is saved as high-quality Markdown files with frontmatter metadata.


## 🚀 Installation

### Using Homebrew

```bash
brew install shayne-snap/tap/waylog
```

### Using Scoop (Windows)

```powershell
scoop bucket add waylog https://github.com/shayne-snap/scoop-bucket
scoop install waylog
```

### Using Cargo

```bash
cargo install waylog
```


## 💡 Usage

### 1. Real-time Logging (`run`)

Use `waylog run` instead of calling your AI tool directly. WayLog will launch the agent and record the conversation in real-time.



```bash
# Run Claude Code with auto-sync
waylog run claude

# Run Gemini CLI
waylog run gemini

# Run Codex CLI
waylog run codex
```

![WayLog Run Demo](demo/run.gif)


### 2. Full Sync / Recover History (`pull`)

Scans your local AI provider storage and "pulls" all relevant sessions into your project's `~/.waylog/history/<project_name>` folder.



```bash
# Pull all history for the current project
waylog pull
```
![WayLog Pull Demo](demo/pull.gif)

## 📂 Supported Providers

| Provider | Status | Description |
|----------|--------|-------------|
| **Claude Code** | 🚧 Beta | Supports `claude` CLI tool from Anthropic. |
| **Gemini CLI** | 🚧 Beta | Supports Google's Gemini CLI tools. |
| **Codex** | 🚧 Beta | Supports OpenAI Codex CLI. |

### Dev build

```bash
git clone https://github.com/shayne-snap/waylog-cli.git
cd waylog-cli
./scripts/install.sh
```


## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

Distributed under the Apache License 2.0. See `LICENSE` for more information.
