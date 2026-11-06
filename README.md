# pipe

Shuyra 的管道服务，提供 Unit 声明、wire API、客户端与常驻 server。

| 目录 | 职责 |
| --- | --- |
| `api/` | marks、操作码、失败码与 wire 帧 |
| `client/` | 请求、owner lease 与映射端口 |
| `server/` | 管道账本与常驻 `pipe` 程序；`unit.toml` 是动态 Unit 入口 |
| `declaration/` | 既有 `UNIT` 与镜像构建坐标 |
| `tests/` | wire、声明与真实账本模块的宿主测试 |
| `tests/roundtrip/` | 动态 Unit 最小往返 guest 与测试声明 |
| `integration/` | 独立的多 Unit 集成工作区 |

创建者持有 lease 并授权端点绑定。读写角色预留持续到管道生命周期结束；
CLOSE 是不可逆的半关闭，不等于 RELEASE 或允许重绑。端点死亡关闭对应方向，
显式 release 或 owner lease 死亡通过 Kernel 撤销根资源。
请求被接纳但回复丢失时，操作可能已提交；BIND 不提供额外幂等保证。
调用者仍负责失败路径上的自身 lease 清理。

## 工具链与维护

使用 `rust-toolchain.toml` 固定的 `nightly-2026-09-04`，首次运行需要 Rustup
下载工具链及 RV64 目标。依赖版本以 Cargo 清单和锁文件为准。
下列命令从仓库根目录执行；宿主测试不代表真实内核运行验证。

```sh
cargo build --locked -p pipe-server --bin pipe --target riscv64gc-unknown-none-elf --release
cargo test --locked --manifest-path tests/Cargo.toml --target x86_64-unknown-linux-gnu
cargo build --locked --manifest-path tests/roundtrip/Cargo.toml --target riscv64gc-unknown-none-elf
```

动态测试由 Vanllia 发现 `tests/roundtrip/test.toml`，使用 System 原生退出与回收回执判定。
它验证绕回、部分 I/O、EOF、显式释放及仍持有端点的能力撤销；失败码为
`0x500000 | (step << 8) | Fail::code()`，步骤见 `tests/roundtrip/src/step.rs`。

[独立集成说明](integration/README.md) 描述另外的多核组合及验收条件。
该工作区依赖固定 Kernel 子模块，使用前执行 `git submodule update --init --recursive`。

## 贡献与来源

维护与 PR 规则见 [Shuyra 贡献规范](https://github.com/shuyra-arche/vanllia/blob/main/CONTRIBUTING.md)。
来源、作者与许可情况见 [PROVENANCE.md](PROVENANCE.md)。
