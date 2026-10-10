_help:
    just -l

# Run the unit tests. The crate has none yet; the ABI glue is checked by `check-example`.
test:
    cargo nextest run --no-tests=pass

# Run the nightly formatter on the crate and the example.
fmt:
    cargo +nightly fmt
    cd examples/bitcrush && cargo +nightly fmt

# Run every check locally. Requires nightly (for rustfmt) and the ARM toolchain.
ci: test check-example
    cargo +nightly fmt --check
    cargo clippy --all-targets
    cargo test --doc

# Build the bitcrush example for the Endless pedal
build-bitcrush:
    cd examples/bitcrush && cargo build --release

# Lint and format-check the example for the ARM target, then verify the emitted
# patch header against the ABI: 136 bytes, 'PTCH' magic, ABI version 0x000B,
# and the Thumb bit set on all thirteen function addresses.
check-example: build-bitcrush
    #!/usr/bin/env bash
    set -euo pipefail
    cd examples/bitcrush
    cargo +nightly fmt --check
    cargo clippy --release
    elf=target/thumbv7em-none-eabihf/release/bitcrush
    header="$elf.header.bin"
    arm-none-eabi-objcopy -O binary --only-section=.patch_header "$elf" "$header"
    size=$(wc -c < "$header" | tr -d ' ')
    if [ "$size" != "136" ]; then
        echo "patch header is $size bytes, expected 136" >&2
        exit 1
    fi
    prefix=$(xxd -p -l 8 "$header")
    if [ "$prefix" != "505443480b000000" ]; then
        echo "patch header starts with $prefix, expected 505443480b000000 (PTCH, ABI 0x000B)" >&2
        exit 1
    fi
    for offset in $(seq 8 4 56); do
        low=$(xxd -p -s "$offset" -l 1 "$header")
        if (( (0x$low & 1) == 0 )); then
            echo "function address at header offset $offset lacks the Thumb bit" >&2
            exit 1
        fi
    done
    echo "patch header OK: $size bytes, magic and ABI version match, Thumb bits set"

# Build a .endl binary from the bitcrush example
endl: build-bitcrush
    @mkdir -p examples/bitcrush/build
    arm-none-eabi-objcopy -O binary \
        examples/bitcrush/target/thumbv7em-none-eabihf/release/bitcrush \
        examples/bitcrush/build/patch_$(date +"%Y%m%d_%H%M%S").endl
    @echo "Built:" examples/bitcrush/build/*.endl

# Inspect the ELF sections of the built bitcrush example
inspect: build-bitcrush
    arm-none-eabi-objdump -h examples/bitcrush/target/thumbv7em-none-eabihf/release/bitcrush

# Install required tools: the ARM toolchain, nextest, release helpers, nightly rustfmt, the ARM target.
setup:
    brew install arm-none-eabi-gcc cargo-nextest
    brew tap ceejbot/tap
    brew install tomato semver-bump
    rustup install nightly
    rustup target add thumbv7em-none-eabihf

# Tag a new version for release.
version BUMP:
    #!/usr/bin/env bash
    set -e
    current=$(tomato get package.version Cargo.toml)
    version=$(semver-bump {{ BUMP }} "$current")
    tomato set package.version "$version" Cargo.toml &> /dev/null
    cargo generate-lockfile
    git commit Cargo.toml -m "v${version}"
    git tag "v${version}"
    echo "Release tagged for version v${version}"

# publish to crates.io
release:
    cargo publish
