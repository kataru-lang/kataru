use crate::traits::{LoadYaml, SaveMessagePack, SaveYaml};
use crate::{
    structs::{Bookmark, Story},
    ParseError,
};

use std::path::Path;

// Dumps the file to disk.
fn dump<S: SaveYaml + SaveMessagePack>(obj: &S, outpath: &Path) -> Result<(), ParseError> {
    // Dump yaml for debugging
    if cfg!(debug_assertions) {
        obj.same_yml(outpath.with_extension("yml"))?;
    }

    // Dump message pack
    obj.save_mp(outpath)
}

/// Parses the config and story files into RMP and writes to the output.
pub fn pack(dir: &str, outdir: &str) -> Result<(), ParseError> {
    let path = Path::new(dir);
    let outpath = Path::new(outdir);

    let story = Story::load_yml(&path.join("story"))?;
    dump(&story, &outpath.join("story"))?;

    // Copy default configs to bookmark.
    let mut bookmark = Bookmark::load_yml(&path.join("bookmark.yml"))?;
    bookmark.init_state(&story);
    dump(&bookmark, &outpath.join("bookmark"))?;
    Ok(())
}
