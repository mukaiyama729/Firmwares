# robot-firmware

Embedded firmware workspace for Pico 2 and other microcontrollers. Protocol
types are maintained in the separate `robot-protocol` repository.

`pico-motor/Cargo.toml` expects the protocol checkout at `../protocol` when
both repositories are cloned independently:

```text
dev/
  firmware/   # this repository
  protocol/   # robot-protocol
```

From the main `MiniAutomobile` repository, the same relative path is provided by
the `crates/robot-adapter/firmware` and `crates/robot-adapter/protocol`
submodules.

Build for Pico 2 with:

```sh
rustup target add thumbv8m.main-none-eabihf
cargo build --release --manifest-path pico-motor/Cargo.toml \
  --target thumbv8m.main-none-eabihf
```

To flash a device, use the existing `probe-rs` runner with `cargo run --release`
from `pico-motor` and connect the Pico 2 by USB or a Debug Probe.
