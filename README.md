# endless-rs

A Rust SDK for the [Polyend Endless](https://polyend.com/endless/) effects pedal. Write custom audio effects in Rust instead of C++. The crate provides a safe `Patch` trait and a macro that generates all the ABI glue needed to produce `.endl` patch binaries.

The [official C/C++ SDK](https://github.com/polyend/FxPatchSDK) is here on GitHub for comparison.

## Requirements

- a stable Rust, edition 2024
- the required ARM target: `thumbv7em-none-eabihf`
- the GNU ARM toolchain: `arm-none-eabi-objcopy` (for producing `.endl` binaries)

```sh
rustup target add thumbv7em-none-eabihf
brew install arm-none-eabi-gcc    # macOS; use your package manager elsewhere
just setup                        # if you use just: installs both of the above plus dev tools
```

See [`examples/bitcrush/`](examples/bitcrush/) for a port of the official SDK's bitcrush effect. If you can build and flash this successfully, you're in business.

### The Patch trait

Write effects by defining a struct to hold any data you need for state, then implementing the `Patch` trait for your struct type. All functions defined by `Patch` have default no-op implementations. Override only what you need.

| Method                                                    | Called                                       |
| --------------------------------------------------------- | -------------------------------------------- |
| `init(&mut self)`                                         | once when patch loads                        |
| `set_working_buffer(&mut self, &'static mut [f32])`       | firmware provides external RAM buffer (once) |
| `process_audio(&mut self, &mut [f32], &mut [f32]) -> f32` | for every audio frame (48 kHz, stereo)       |
| `param_metadata(&self, ParamId) -> ParameterMetadata`     | when firmware queries knob ranges            |
| `set_param(&mut self, ParamId, f32)`                      | when user turns a knob (audio thread)        |
| `handle_action(&mut self, ActionId)`                      | when the foot switch is pressed or held      |
| `state_led_color(&self) -> Color`                         | the firmware polls LED color                 |

### Constraints

Endless effects are `#![no_std]` code that runs on an ARM Cortex-M7. Keep in mind the limitations.

- **No heap allocation.** Use static storage or the working buffer.
- **Single-precision float only.** Use `f32` everywhere; `f64` operations are software-emulated and slow.
- **Keep `process_audio` fast.** If it takes too long, audio frames drop.
- **Output samples in (-1.0, 1.0)** to avoid digital clipping.
- For math functions like `powf` or `sinf`, add the [`libm`](https://crates.io/crates/libm) crate.

## Write your own Endless effect

Here's another full example of making a new binary crate that depends on `endless-rs`:

```toml
# Cargo.toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "hello-world"
path = "src/main.rs"

[dependencies]
endless-rs = "0.2"

[profile.release]
opt-level = "s"
lto = true
strip = "symbols"
```

Add a `.cargo/config.toml` to target the Endless hardware. The `endless-rs` build script puts its linker script on the search path, so `-Tpatch_imx.ld` resolves wherever the crate lives:

```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
rustflags = [
  "-C", "link-arg=-Tpatch_imx.ld",
  "-C", "link-arg=--defsym=PATCH_LOAD_ADDR=0x80000000",
]
```

And now we write the hello world of DSP effects: gain!

```rust
#![no_std]
#![no_main]

use endless_rs::{ParamId, Patch, endless_patch};

struct HelloWorld {
    gain: f32,
}

impl Patch for HelloWorld {
    fn process_audio(&mut self, left: &mut [f32], right: &mut [f32]) -> f32 {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= self.gain;
            *r *= self.gain;
        }
        0.0
    }

    fn set_param(&mut self, param: ParamId, value: f32) {
        if param == ParamId::Left {
            self.gain = value;
        }
    }
}

endless_patch!(HelloWorld, HelloWorld { gain: 0.5 });
```

Build the Rust binary, then run the ARM tool that turns it into a flashable artifact:

```sh
cargo build --release
arm-none-eabi-objcopy -O binary \
    target/thumbv7em-none-eabihf/release/hello-world \
    hello-world.endl
```

Connect the Endless pedal to your computer over USB C, and copy the `.endl` file to the mounted Endless drive. If all goes well, the pedal's LED turns yellow, then flashes green, then becomes a steady blue, indicating that the effect is active. If the pedal LED goes red, the flash failed.

## Developing this crate

The crate itself has no unit tests: the ABI glue only exists once the `endless_patch!` macro is expanded, and that happens in the example. `just ci` runs every check, including building the example for the ARM target and verifying the emitted patch header byte for byte against the ABI.

```sh
just ci      # fmt check, clippy, doctests, and the ARM build + header check of the example
just endl    # build the bitcrush example into a timestamped .endl file
just inspect # dump the example's ELF sections
```

## License

MIT
