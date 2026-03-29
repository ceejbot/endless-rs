_help:
    just -l

# Run all tests using nextest.
test:
    cargo nextest run

# Run the nightly formatter
fmt:
    cargo +nightly fmt

# Run the same checks we run in CI. Requires nightly.
ci: test fmt
    cargo clippy --all-targets
    cargo test --doc

# Build the bitcrush example for the Endless pedal
build-bitcrush:
    cd examples/bitcrush && cargo build --release

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

# Install required tools
setup:
    brew tap ceejbot/tap
    brew install cargo-nextest tomato semver-bump
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
