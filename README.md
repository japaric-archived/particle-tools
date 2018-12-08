# Status

This crate is **UNMAINTAINED**. I no longer own a Photon board and won't be able
to work on this.

-- @japaric, 2018-12-08

# `particle-tools`

> Tools to aid with development of [Particle] applications

[Particle]: https://particle.io

# Tools

- `elf2bin`. A tool to convert an ELF file into a binary file compatible with
  the `particle flash` command.

# Usage

```
$ cargo install --git https://github.com/japaric/particle-tools

$ elf2bin target/photon/release/examples/blinky

$ particle flash $device blinky.bin
```

# License

The Rust code in repository is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

The binary blobs in this repository were generated from the [spark/firmware]
repository and as such they are licensed according to [their terms].

[spark/firmware]: https://github.com/spark/firmware/tree/v0.6.2
[their terms]: https://github.com/spark/firmware/tree/v0.6.2#license

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
