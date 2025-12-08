use std::{fs, io, path::Path};

pub fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        // Same error as `fs::read_to_string` produces (using `io::ErrorKind::InvalidData`)
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}
