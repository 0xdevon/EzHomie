# EzHomie

[English](#english) | [中文](#中文)

## English

EzHomie is a macOS-first command line app for starting and ending a workday.

```bash
ezhomie on
ezhomie off
```

`ezhomie on` opens configured apps, URLs, and shell commands in order.
`ezhomie off` gently quits configured apps, then asks before shutting down.

### Install Rust

This repository is a Rust project. If `cargo` is not installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart the shell or source Cargo's environment file:

```bash
source "$HOME/.cargo/env"
```

### Build

```bash
cargo build
cargo test
```

### Configure

Create the default config:

```bash
cargo run -- config init
```

The default config path is:

```text
~/.config/ezhomie/config.toml
```

Example:

```toml
[off]
confirm_seconds = 10
shutdown = true
quit_timeout_secs = 15

[[apps]]
name = "Cursor"
delay_ms = 500

[[apps]]
name = "Google Chrome"
delay_ms = 500

[[urls]]
url = "https://calendar.google.com"
delay_ms = 300

[[commands]]
name = "start dev service"
cmd = "cd ~/project && npm run dev"
delay_ms = 0
```

### Usage

Preview the morning plan:

```bash
cargo run -- on --dry-run
```

Preview the evening plan:

```bash
cargo run -- off --dry-run
```

Quit configured apps without shutting down:

```bash
cargo run -- off --no-shutdown
```

Skip the shutdown countdown:

```bash
cargo run -- off --yes
```

Use another config file:

```bash
cargo run -- --config ./config.toml status
```

## 中文

EzHomie 是一个优先支持 macOS 的命令行应用，用来一键开始或结束工作日。

```bash
ezhomie on
ezhomie off
```

`ezhomie on` 会按顺序打开配置好的 App、URL 和 shell 命令。
`ezhomie off` 会温和退出配置好的 App，并在关机前进行确认。

### 安装 Rust

本仓库是一个 Rust 项目。如果还没有安装 `cargo`：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

然后重启 shell，或加载 Cargo 的环境文件：

```bash
source "$HOME/.cargo/env"
```

### 构建

```bash
cargo build
cargo test
```

### 配置

创建默认配置：

```bash
cargo run -- config init
```

默认配置路径：

```text
~/.config/ezhomie/config.toml
```

示例：

```toml
[off]
confirm_seconds = 10
shutdown = true
quit_timeout_secs = 15

[[apps]]
name = "Cursor"
delay_ms = 500

[[apps]]
name = "Google Chrome"
delay_ms = 500

[[urls]]
url = "https://calendar.google.com"
delay_ms = 300

[[commands]]
name = "start dev service"
cmd = "cd ~/project && npm run dev"
delay_ms = 0
```

### 使用

预览上班启动计划：

```bash
cargo run -- on --dry-run
```

预览下班关闭计划：

```bash
cargo run -- off --dry-run
```

只退出配置好的 App，不关机：

```bash
cargo run -- off --no-shutdown
```

跳过关机倒计时：

```bash
cargo run -- off --yes
```

使用其他配置文件：

```bash
cargo run -- --config ./config.toml status
```
