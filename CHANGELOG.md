# Changelog

All notable changes to this project should be documented in this file.
Departures from this policy are bugs; feel free to file an issue. This project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) with
regard to the `moodly` binary's behavior (not the code library).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Planned]

- subcommands
  - `clean`

## [2.0.0] - 2025-06-17

### Fixed

- dates are now saved as `%Y-%m-%d` instead of `%Y%m%d`
- times are now saved as `%H:%M` instead of `%H%M`
- manual time inputs no longer always fail
- remove redundant `to_string`

## [1.1.0] - 2025-06-17

### Added

- `run` method to `Cli` to make main file smaller
- subcommands
  - `tail`
  - `dump`
  - `where`

## [1.0.0] - 2025-06-16

### Added

- `moodly` binary
  - no flags or subcommands
