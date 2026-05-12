# Release And macOS Distribution Plan

The current repository can be installed directly with Cargo:

```bash
cargo install --git https://github.com/3873225350/make-agents-cheaper.git
```

## Recommended Release Path

1. Tag a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

2. Build binaries for common targets. The repository includes a tag-triggered workflow:

```text
.github/workflows/release.yml
```

It builds:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

For a local build:

```bash
cargo build --release
```

3. Attach binaries and checksums to a GitHub Release.

4. Add a Homebrew tap after the first release artifact exists. A formula template is tracked at:

```text
packaging/homebrew/make-agents-cheaper.rb
```

Do not publish a Homebrew formula until the release tag, tarball checksum, and install test are stable.
