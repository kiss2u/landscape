# Landscape Router
使用 eBPF 技术,以及 AF_PACKET. 通过网页操作将 Linux 配置成 路由.

目前的功能都是使用 Rust 进行实现的, 不是进行调用 PPPoE DHCP 等现有程序

## 启动方式以及限制
目前功能所需的 Linux 版本需要在 `6.1` 及以上

启动前需要先在 用户的 `home` 目录创建 
```shell
mkdir -p ~/.landscape-route
```

然后将 `geosite.dat` 文件, 放入该文件夹.

### 编译
安装所需工具
```shell
apt install pkg-config bpftool build-essential clang libelf1 libelf-dev zlib1g-dev
```

```
cargo build --release
```
产物将会在 `target/release/landscape-webserver`

运行即可
```shell
./landscape-webserver
```
