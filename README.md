# ydcv-rs

[![Build Status](https://travis-ci.org/farseerfc/ydcv-rs.svg)](https://travis-ci.org/farseerfc/ydcv-rs)

A rust version of [ydcv](https://github.com/felixonmars/ydcv/).

# How to build

Can use `curl-rust` (default) or `hyper` as HTTP client library.

To use `curl-rust`, which will have 21 dependencies, build with:
```bash
cargo build
```

To use `hyper`, which will have 36 dependencies, build with:
```bash
cargo build --no-default-features --features "use_hyper"
```

# (Original) YouDao Console Version

Simple wrapper for Youdao online translate (Chinese <-> English) service [API](http://fanyi.youdao.com/openapi?path=data-mode), as an alternative to the StarDict Console Version(sdcv).