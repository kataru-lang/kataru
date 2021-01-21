use crate::structs::{Bookmark, Story};
use crate::traits::Loadable;

use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

fn dump<S: Serialize>(obj: &S, outpath: &Path) -> io::Result<()> {
    // Dump yaml for debugging
    if cfg!(debug_assertions) {
        let yml_outfile = BufWriter::new(File::create(outpath.with_extension("yml")).unwrap());
        serde_yaml::to_writer(yml_outfile, obj).unwrap();
    }

    let mut outfile = BufWriter::new(File::create(outpath).unwrap());
    outfile.write(&rmp_serde::to_vec(obj).unwrap())?;
    Ok(())
}

/// Parses the config and story files into RMP and writes to the output.
pub fn pack(dir: &str, outdir: &str) -> io::Result<()> {
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
