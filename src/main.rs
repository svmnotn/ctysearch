use edit_distance::edit_distance;
use std::path::PathBuf;

mod error;
mod function;
mod parse;

const EXAMPLE: &[u8] = include_bytes!("../example.h");

fn main() {
    let search = "Tes_t (uint8, uint)";
    let mut fns =
        parse::find_all(PathBuf::from("example.h"), EXAMPLE).expect("Example should work");

    fns.sort_by_cached_key(|f| edit_distance(&f.cannonize(), search));

    for f in fns {
        println!("{f}");
    }
}
