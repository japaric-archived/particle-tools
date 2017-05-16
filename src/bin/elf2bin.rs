extern crate byteorder;
extern crate clap;
extern crate crc;
#[macro_use]
extern crate error_chain;
extern crate sha2;
extern crate tempdir;

mod errors {
    error_chain!();
}

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use byteorder::{BigEndian, ByteOrder};
use clap::{App, Arg};
use crc::crc32;
use sha2::{Digest, Sha256};
use tempdir::TempDir;

use errors::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = App::new("elf2bin")
        .version(env!("CARGO_PKG_VERSION"))
        .about(
            "Converts ELF files into binary files compatible with \
             `particle flash`"
        )
        .arg(
            Arg::with_name("INPUT")
                .help("ELF file to convert")
                .required(true)
                .index(1)
        )
        .get_matches();

    let input = PathBuf::from(matches.value_of("INPUT").unwrap());
    let name = input.file_name().ok_or("input is not a file")?;

    let pre_crc = objcopy(&input)?;
    let post_crc = checksum(&pre_crc)?;

    let outfile = &Path::new(name).with_extension("bin");
    File::create(outfile)
        .chain_err(|| format!("couldn't create file {}", outfile.display()))?
        .write_all(&post_crc)
        .chain_err(|| format!("couldn't write to file {}", outfile.display()))?;

    Ok(())
}

fn objcopy(path: &Path) -> Result<Vec<u8>> {
    let td = TempDir::new("elf2bin")
        .chain_err(|| "couldn't create a temporary directory")?;
    let td = td.path();
    let tmpfile = &td.join("output");

    let output = Command::new("arm-none-eabi-objcopy")
        .args(&["-O", "binary"])
        .arg(path)
        .arg(tmpfile)
        .output()
        .chain_err(|| "couldn't run `arm-none-eabi-objcopy`")?;

    if !output.status.success() {
        bail!(
            "`arm-none-eabi-objcopy` error:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    }

    let mut output = vec![];
    File::open(tmpfile)
        .chain_err(|| format!("couldn't open {}", tmpfile.display()))?
        .read_to_end(&mut output)
        .chain_err(|| format!("error reading {}", tmpfile.display()))?;

    Ok(output)
}

fn checksum(pre_crc: &[u8]) -> Result<Vec<u8>> {
    const MAGIC_SIZE: usize = 38;
    const MAGIC_STRING: &str = "0102030405060708090a0b0c0d0e0f1011121314151617\
                                18191a1b1c1d1e1f20280078563412";

    // First we check that the binary contains our magic string
    // This magic string is at the final `MAGIC_SIZE` bytes of the binary
    let n = pre_crc.len();
    // the split between the real binary and the magic string
    let split = n - MAGIC_SIZE;
    let no_crc = &pre_crc[..split];
    let crc_block = &pre_crc[split..];

    let hexdump = crc_block
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .concat();

    if hexdump != MAGIC_STRING {
        bail!("invalid ELF file; the magic string doesn't match")
    }

    // Then we add a proper checksum to the binary
    // We want to produce something like this: [`no_crc`, `new_crc`]
    // where `no_crc` is the original binary without the magic string
    // `new_crc` will be computed like this:
    // - the first 32 bytes will be the SHA256 of `no_crc`
    // - the next 2 bytes will be the same `MAGIC_STRING[32..34]`
    // - the final 4 bytes will be the CRC32 of
    //   [`no_crc`, `new_crc[..32]`, `MAGIC_STRING[32..34]`]
    let sha = {
        let mut hasher = Sha256::default();
        hasher.input(no_crc);
        hasher.result()
    };

    let mut post_crc = pre_crc.to_owned();
    post_crc[split..(split + 32)].copy_from_slice(&sha);

    let checksum = crc32::checksum_ieee(&post_crc[..(n - 4)]);
    let mut crc32 = [0; 4];
    BigEndian::write_u32(&mut crc32, checksum);
    post_crc[(n - 4)..].copy_from_slice(&crc32);

    Ok(post_crc)
}
