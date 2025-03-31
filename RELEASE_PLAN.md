# Release Workflow Improvement Plan

## Current Issues
1. Uses deprecated actions-rs/toolchain
2. ARM architecture builds failing
3. Complex artifact handling

## Proposed Solution
1. Replace actions-rs with dtolnay/rust-toolchain
2. Support both x86_64 and aarch64 architectures
3. Simplify build and release process

## New Workflow Design
```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        target: 
          - x86_64-unknown-linux-gnu
          - aarch64-unknown-linux-gnu
          - x86_64-apple-darwin
          - aarch64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/release/zone-transfer

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            artifacts/*/zone-transfer