# ![Coupler](assets/coupler-banner.svg)

Coupler is a framework for writing audio plugins in Rust. It currently supports the VST3 and CLAP APIs, with plans to support AUv2 and AAX in the near future.

Like the Rust language itself, Coupler prioritizes:

1. **Performance:** Coupler follows a "pay for what you use" philosophy and strives to impose as little overhead as possible between hosts and plugins.
2. **Correctness:** A plugin API is a contract, with rules for threading and memory management that must be followed by both host and plugin. Coupler uses Rust's type system and ownership model to ensure that plugins implement their end of this contract correctly.
3. **Productivity:** Coupler provides a [Serde](https://crates.io/crates/serde)-like `#[derive(Params)]` macro for declaring plugin parameters and a `cargo coupler bundle` command for generating plugin bundles.

Coupler is still early in development and should not be considered production-ready. Important functionality is still missing and there is little to no documentation. However, feel free to experiment with it. Questions and feedback are welcome on the [Zulip instance](https://coupler.zulipchat.com) or on the [Rust Audio Discord](https://discord.gg/yVCFhmQYPC).

## Building

First, see the usage instructions for the [`vst3` crate](https://github.com/coupler-rs/vst3-rs?tab=readme-ov-file#usage).

To build the `gain` example, run:

```console
cargo run -p cargo-coupler -- coupler bundle -p gain --release
```

VST3 and CLAP plugin bundles will be placed in the `target/release/bundle` directory.

## License

Coupler is distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache license, version 2.0](LICENSE-APACHE). Contributions are accepted under the same terms.
