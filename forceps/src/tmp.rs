use rand::{distributions::Alphanumeric, prelude::*};
use std::cell::UnsafeCell;
use std::ffi::{OsStr, OsString};
use std::path;
use std::str;

std::thread_local! {
    static RNG: UnsafeCell<SmallRng> = UnsafeCell::new(SmallRng::from_entropy());
}

/// Reimplementation of [this][tmpfile]. Licensed under Apache license
///
/// [tmpfile]: https://github.com/Stebalien/tempfile/blob/c361acc1213605eb26be0ad6de4c0f29cb7491f0/src/util.rs#L17
fn tmpname(prefix: &OsStr, rand_len: usize) -> OsString {
    let mut buf = OsString::with_capacity(prefix.len() + rand_len);
    buf.push(prefix);

    // Push each character in one-by-one. Unfortunately, this is the only
    // safe(ish) simple way to do this without allocating a temporary
    // String/Vec.
    RNG.with(|rng| unsafe {
        (&mut *rng.get())
            .sample_iter(&Alphanumeric)
            .take(rand_len)
            .for_each(|b| buf.push(str::from_utf8_unchecked(&[b as u8])))
    });
    buf
}

/// Creates a randomized path in a directory that can be used as a temporary file
pub(crate) fn tmppath_in(dir: &path::Path) -> path::PathBuf {
    const LEN: usize = 10;
    let mut buf = path::PathBuf::new();
    buf.push(dir);
    buf.push(&tmpname(OsStr::new("tmp"), LEN));
    buf
}
