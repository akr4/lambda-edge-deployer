use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, failure::Error>;

pub fn package<P: AsRef<Path>>(path: P) -> Result<tempfile::NamedTempFile> {
    let zip_file = NamedTempFile::new()?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let mut buf = Vec::new();
    BufReader::new(File::open(&path)?).read_to_end(&mut buf)?;

    zip.start_file("index.js", Default::default())?;
    zip.write_all(&buf)?;
    let zip_file = zip.finish()?;

    Ok(zip_file)
}
