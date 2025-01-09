### 如果你需要在 x86 下进行交叉编译
如果你需要进行交叉编译, 可以使用以下步骤

> 目前的步骤仅在 Debian 下进行验证

#### 1. 确保 Rust 工具链支持交叉编译
使用 `rustup` 安装 Rust 工具链并添加 `aarch64` 目标：
```bash
rustup target add aarch64-unknown-linux-gnu
```

#### 2. 安装交叉编译依赖
在进行交叉编译时，Rust 会调用目标架构的链接器，因此需要安装对应的工具链。

```bash
# 启用 ARM64 架构支持
sudo dpkg --add-architecture arm64
sudo apt update

sudo apt install gcc-aarch64-linux-gnu libelf-dev:arm64 zlib1g-dev:arm64
```

**检查安装是否成功**：
```bash
aarch64-linux-gnu-gcc --version
```

#### 3. 配置 Rust 工程的链接器（可选）
为了简化交叉编译的命令，你可以在项目的 `.cargo/config.toml` 中配置默认链接器：

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

#### 4. 进行交叉编译
完成依赖安装后，可以运行以下命令进行交叉编译：

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

