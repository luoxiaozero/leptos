# 开始

这里有两个开始学习 Lepots 的基本路线。

1. 利用 [Trunk](https://trunkrs.dev/) 的客户端渲染。
2. 利用 [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos) 的全栈渲染。

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

> 使用 `nightly` Rust 和 leptos 的 `nightly` 特征，可以启用 signal 的 getters 和 setters 函数调用语法。在本书中大多数使用了该方法。
>
> 使用 `nightly` Rust, 你可以运行
>
> ```bash
> rustup toolchain install nightly
> rustup default night
> ```
>
> 如果你更愿意使用 stable Rust 和 Leptos，你也可以这样做，在指南和例子中你只需要使用[`ReadSignal::get()`](https://docs.rs/leptos/latest/leptos/struct.ReadSignal.html#impl-SignalGet%3CT%3E-for-ReadSignal%3CT%3E) 和 [`WriteSignal::set()`](https://docs.rs/leptos/latest/leptos/struct.WriteSignal.html#impl-SignalGet%3CT%3E-for-ReadSignal%3CT%3E) 方法，而不是作为函数调用 signal 的 getters 和 setters。

确保你已经添加 `wasm32-unknown-unknown` 目标，以便 Rust 能编译你的代码到 WebAssembly 已在游览器运行。

```bash
rustup target add wasm32-unknown-unknown
```

在 `leptos-tutorial` 的根目录创建一个简单的 `index.html`

```html
<!DOCTYPE html>
<html>
  <head></head>
  <body></body>
</html>
```
