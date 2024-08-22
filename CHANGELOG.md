# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.5.0] - 2024-08-22

### Added

- Health check capabilities

## [2.4.0] - 2024-08-20

### Added

- recover from state where lazymc is unable to stop the server as the server started without lazymc starting it
- test which ensures it can recover from an unexpected state
- test which uses a minecraft bot to login to the server to ensure it functions correctly

### Changed

- On boot, the app will stop all server containers to ensure state is controlled
- On shutdown, the app stop all server containers to ensure state is controlled

### Fixed

- Resolve static ip for container if not using IPAM also
- Warns when the container IP could not be resolved (only an issue if not using IPAM config)

## [2.3.0] - 2024-08-18

### Fixed

- Use static IP in the lazymc.toml instead of hostname

### Deprecated

- The app will throw a warning if it notices you are not using a static ip for the minecraft container

## [2.2.0] - 2024-08-05

### Added

- support for multiple minecraft containers
- `lazymc.port` label and `LAZYMC_PORT` env var
- `lazymc.server.directory` label and `LAZYMC_SERVER_DIRECTORY` env var
- wrapped lazymc logs to add relevant `lazymc.group` information

## [2.1.0] - 2024-08-03

### Added

- use container labels on the minecraft server to configure `lazymc-docker-proxy`

### Deprecated

- using Environment Variables to configure lazymc is deprecated. This change is vital for handling multiple servers in the future.

## [2.0.2] - 2024-08-02

### Fixed

- legacy lazymc version is now set correctly in lazymc.toml

## [2.0.1] - 2024-07-31

### Build

- docker image optimization, now using scratch as base

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

[unreleased]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.5.0...HEAD
[2.5.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.4.0...v2.5.0
[2.4.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.0.2...v2.1.0
[2.0.2]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/joesturge/lazymc-docker-proxy/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v1.0.0...v2.0.0
[1.0.0]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.6...v1.0.0
[0.1.6]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joesturge/lazymc-docker-proxy/compare/v0.1.0...v0.1.1
