use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

fn read(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

pub fn pack(dir: &str) -> io::Result<()> {
    let dir_path = Path::new(dir);
    let mut contents = String::new();
    for path in fs::read_dir(dir_path)? {
        contents.push_str(&read(&path?.path())?);
    }

    let mut file = File::create(dir_path.join("../.passages.yml"))?;
    file.write_all(contents.as_bytes())
}
