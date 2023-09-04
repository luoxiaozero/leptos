# 开始

这里有两个开始学习 Lepots 的基本路径。

1. [Trunk](https://trunkrs.dev/) 的客户端渲染。
2. [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos) 的全栈渲染。

对于早期的例子，我们从 Trunk 开始比较容易入门，稍后我们在介绍 `cargo-leptos`。

如果你还没安装 Trunk，你可以运行以下命令安装

```bash
cargo install trunk
```

创建一个基本的 Rust 项目

```bash
cargo init leptos-tutorial
```

`cd` 进入你新建的 `leptos-turorial` 项目，并且添加 `leptos` 到项目依赖里

```bash
cargo add leptos --features=csr,nightly
```
或者你如果用的是 stable 的 Rust，你可以移除 `nightly`

```bash
cargo add leptos --features=csr
```
