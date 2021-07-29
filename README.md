# SOCKS toolkit
[![Build Status](https://github.com/onnovalkering/socksx/workflows/CI/badge.svg)](https://github.com/onnovalkering/socksx/actions)
[![License: MIT](https://img.shields.io/github/license/onnovalkering/socksx.svg)](https://github.com/onnovalkering/socksx/blob/master/LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/onnovalkering/socksx/badge.svg)](https://coveralls.io/github/onnovalkering/socksx?branch=master)
[![Crates.io](https://img.shields.io/crates/v/socksx)](https://crates.io/crates/socksx)

A work-in-progress SOCKS toolkit for Rust. SOCKS5 ([rfc1928](https://tools.ietf.org/html/rfc1928)) and SOCKS6 ([draft-11](https://tools.ietf.org/html/draft-olteanu-intarea-socks-6-11)) are supported.

[Documentation](https://docs.rs/socksx/latest)

## Python
The `socksx-py` crate is a [PyO3](https://github.com/PyO3/PyO3)-based Python interface to `socksx`.

You can build and install this Python package locally (requires [`pipenv`](https://github.com/pypa/pipenv) and [`maturin`](https://github.com/PyO3/maturin)):

```shell
$ pipenv install && pipenv shell
$ maturin develop -m ./socksx-py/Cargo.toml
```

To build a manylinux releases:

```shell
$ docker run --rm -v $(pwd):/io konstin2/maturin build --release -m ./socksx-py/Cargo.toml
```