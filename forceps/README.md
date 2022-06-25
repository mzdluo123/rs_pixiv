<div align="center">

# Forceps
[![docs.rs][docs-rs-badge]][docs-rs-url]
[![crates.io][crates-badge]][crates-url]
[![CI][ci-badge]][ci-url]

**The easy-to-use and asynchronous solution for your tokio project**

[docs-rs-badge]: https://docs.rs/forceps/badge.svg
[docs-rs-url]: https://docs.rs/forceps/*/forceps
[crates-badge]: https://img.shields.io/crates/v/forceps.svg
[crates-url]: https://crates.io/crates/forceps
[ci-badge]: https://github.com/blockba5her/forceps/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/blockba5her/forceps/actions/workflows/ci.yml

</div>

---

`forceps` is made to be an easy-to-use, thread-safe, performant, and asynchronous disk cache
that has easy reading and manipulation of data. It levereges tokio's async `fs` APIs
and fast task schedulers to perform IO operations, and `sled` as a fast metadata database.

It was originally designed to be used in [`scalpel`](https://github.com/blockba5her/scalpel),
the MD@Home implementation for the Rust language.

## Instability Warning

Just as a **warning**, this crate is still yet to be heavily tested and is still lacking features.
It is advisable to use another solution if you have the option!

## Features

- Asynchronous APIs
- Fast and reliable reading/writing
- Tuned for large-file databases
- Included cache eviction (LRU/FIFO)
- Easily accessible value metadata
- Optimized for cache `HIT`s
- Easy error handling
- `bytes` crate support (non-optional)

### Planned Features

- Toggleable in-memory LRU cache
- Optional tracking of last-access timestamps
- Built-in cache integrity checks

## Documentation

All documentation for this project can be found at [docs.rs](https://docs.rs/forceps/*/forceps).

## License

This project is licensed under the `MIT` license. Please see
[LICENSE](https://github.com/blockba5her/forceps/blob/main/LICENSE) for more information.
