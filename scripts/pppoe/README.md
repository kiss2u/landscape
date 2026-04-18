# PPPoE Test Scripts

这组脚本用于在本机创建 `netns + veth + pppoe-server` 的原生 PPPoE 联调环境，并执行一次自动化 smoke test。

## 前置依赖

需要 root 权限，并确保系统已安装：

- `ip`
- `pppd`
- `pppoe-server`
- Rust/Cargo 可用

## 脚本

- `pppoe_test_env.sh`
  - 管理测试环境
  - 子命令：`up` `down` `status`
- `pppoe_test_run.sh`
  - 一键拉起环境并执行 PPPoE smoke test

## 一键测试

```bash
sudo ./scripts/pppoe/pppoe_test_run.sh
```

保留现场：

```bash
sudo ./scripts/pppoe/pppoe_test_run.sh --keep-env
```

## 分步使用

启动环境：

```bash
sudo ./scripts/pppoe/pppoe_test_env.sh up
```

查看状态：

```bash
sudo ./scripts/pppoe/pppoe_test_env.sh status
```

清理环境：

```bash
sudo ./scripts/pppoe/pppoe_test_env.sh down
```

## 默认测试参数

- namespace: `ld-pppoe-test`
- client iface: `ld-pppoe-client`
- server iface: `ld-pppoe-server`
- username: `pppoe-user`
- password: `pppoe-pass`
- local IP: `10.0.0.1`
- remote IP start: `10.0.0.100`
- MTU: `1492`

## 可覆盖的环境变量

- `PPPOE_TEST_NS`
- `PPPOE_TEST_CLIENT_IFACE`
- `PPPOE_TEST_SERVER_IFACE`
- `PPPOE_TEST_USERNAME`
- `PPPOE_TEST_PASSWORD`
- `PPPOE_TEST_LOCAL_IP`
- `PPPOE_TEST_REMOTE_IP_START`
- `PPPOE_TEST_TIMEOUT_SECS`
- `PPPOE_TEST_MTU`

示例：

```bash
sudo PPPOE_TEST_USERNAME=user1 PPPOE_TEST_PASSWORD=pass1 ./scripts/pppoe/pppoe_test_run.sh
```

## 日志

运行时文件默认写到：

```text
/tmp/landscape-pppoe-test
```

主要文件：

- `pppoe-server.log`
- `pppd.log`
- `pppoe-client.stdout.log`
- `pppoe-client.stderr.log`

## 成功判定

当前 smoke test 以客户端进入 `Running` 为成功标准。

服务端侧通常可以在 `pppd.log` 中看到：

- `PAP peer authentication succeeded`
- `local IP address ...`
- `remote IP address ...`
