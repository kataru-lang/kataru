use crate::traits::Loadable;
use crate::{
    structs::{Bookmark, Story},
    ParseError,
};

use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

// Dumps the file to disk.
fn dump<S: Serialize>(obj: &S, outpath: &Path) -> Result<(), ParseError> {
    // Dump yaml for debugging
    if cfg!(debug_assertions) {
        let file = match File::create(outpath.with_extension("yml")) {
            Ok(f) => f,
            Err(e) => return Err(perror!("Failed to create file: {:?}", e)),
        };
        let yml_outfile = BufWriter::new(file);
        match serde_yaml::to_writer(yml_outfile, obj) {
            Ok(_) => (),
            Err(e) => return Err(perror!("Failed to write to file: {:?}", e)),
        }
    }

    // Dump message pack
    let file = match File::create(outpath) {
        Ok(f) => f,
        Err(e) => return Err(perror!("Failed to create file: {:?}", e)),
    };
    let mut outfile = BufWriter::new(file);
    let buffer = match rmp_serde::to_vec(obj) {
        Ok(b) => b,
        Err(e) => return Err(perror!("Failed to serialize object: {:?}", e)),
    };
    match outfile.write(&buffer) {
        Ok(_) => Ok(()),
        Err(e) => return Err(perror!("Error writing MessagePack buffer: {:?}", e)),
    }
}

/// Parses the config and story files into RMP and writes to the output.
pub fn pack(dir: &str, outdir: &str) -> Result<(), ParseError> {
    let path = Path::new(dir);
    let outpath = Path::new(outdir);

    let story = Story::load(&path.join("story"))?;
    dump(&story, &outpath.join("story"))?;

    // Copy default configs to bookmark.
    let mut bookmark = Bookmark::load(&path.join("bookmark.yml"))?;
    bookmark.init_state(&story);
    dump(&bookmark, &outpath.join("bookmark"))?;
    Ok(())
}
