# Contributing to lazymc-docker-proxy

Thanks for helping improve this project! Contributions are welcome, including bug reports, documentation fixes, tests, and code changes.

## Report a bug

Before opening an issue, search existing issues first.

Please include:
- a short description of the problem
- expected vs actual behavior
- reproduction steps
- versions used (Docker, Docker Compose, Rust, OS, Minecraft server image)
- relevant logs or error output

If the issue is security-sensitive, do not post it publicly.

## Suggest an enhancement

Search existing issues before opening a new one.

When suggesting a feature or improvement, include:
- current behavior
- desired behavior
- why the change is useful
- any relevant setup or configuration

Small documentation or setup improvements are also welcome.

## Development

Required tools:
- Rust
- Docker and Docker Compose
- `bats-core` for integration tests

Build locally:

```bash
cargo build --release
```

Build the Docker image:

```bash
docker build .
```

Tests:
- Integration tests are in `tests/bats`
- GitHub Actions run these tests on each PR

If your change affects users, update `CHANGELOG.md` under the `[Unreleased]` section.

## Commit messages

Prefer [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

### Changelog

This project uses [keepachangelog](https://keepachangelog.com/en/1.1.0/) to maintain it's changelog.
If you are proposing a change please update the [changelog](CHANGELOG.md) by adding your changes under the `[Unreleased]` header
