### If You Need to Perform Cross-Compilation on x86

If you need to perform cross-compilation, you can follow the steps below.

> The current steps are only verified on Debian.

#### 1. Ensure Rust Toolchain Supports Cross-Compilation

Use `rustup` to install the Rust toolchain and add the `aarch64` target:

```bash
rustup target add aarch64-unknown-linux-gnu
```

#### 2. Install Cross-Compilation Dependencies

During cross-compilation, Rust will invoke the linker for the target architecture. Therefore, you need to install the corresponding toolchain.

```bash
# Enable ARM64 architecture support
sudo dpkg --add-architecture arm64
sudo apt update

sudo apt install gcc-aarch64-linux-gnu libelf-dev:arm64 zlib1g-dev:arm64
```

**Verify Installation:**
```bash
aarch64-linux-gnu-gcc --version
```

#### 3. Configure Rust Project Linker (Optional)

To simplify the cross-compilation command, you can configure the default linker in the project's `.cargo/config.toml` file:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

#### 4. Perform Cross-Compilation

After installing the dependencies, you can run the following command to perform cross-compilation:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```
