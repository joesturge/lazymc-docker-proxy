# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2024-07-30

### Added

- support for minecraft client versions older than 1.20.3
- test for forge server using 1.19.2 client version

### Removed

- RCON configuration options as this breaks the SIGTERM signal being received

## [1.0.0] - 2024-07-29

### Added

- bats integration test
- contributing guide

### Fixed

- Integration tests
- Inconsistencies in the docker compose examples

## [0.1.6] - 2024-07-28

### Fixed

- using latest version of thomaseizinger/keep-a-changelog-new-release

## [0.1.5] - 2024-07-28

### Removed

- removed creating releases, they are not needed anyway

## [0.1.4] - 2024-07-28

### Fixed

- release notes no longer missing

## [0.1.3] - 2024-07-28

### Fixed

- tag name generation for changelogs

## [0.1.2] - 2024-07-28

### Added

- changelog generation
- github release generation

## [0.1.1] - 2024-07-27

### Fixed

- Tag release job

## 0.1.0 - 2024-07-27

### Added

- Rust implementation of lazymc-docker-proxy
- Detailed README file with example of usage and env var description
- Github actions to build and push docker image

[unreleased]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v1.0.0...v2.0.0
[1.0.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.6...v1.0.0
[0.1.6]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.0...v0.1.1
