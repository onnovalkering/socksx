# Changelog

All notable changes to `socksx` will be documented in this file.

## [0.1.1] - 2021-07-29
### Added
- Experimental support for chaining (SOCKS6).
- Python package (`socksx-py`) with interface to `socksx`.

### Changed
- Use `tokio::io::copy_bidirectional` instead of local copy.

## [0.1.0] - 2021-03-16
### Added
- Initial implementation.