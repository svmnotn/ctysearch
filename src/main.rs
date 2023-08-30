use edit_distance::edit_distance;
use std::fs::read;

mod args;
mod error;
mod function;
mod parse;

fn main() -> Result<(), error::Error> {
    let args = args::parse_args()?;

    let search = args.search.as_str();
    let path = args.header.as_path();
    let text = read(path)?;
    let mut fns = parse::find_all(path, &text)?;

    fns.sort_by_cached_key(|f| edit_distance(&f.canonicalize(), search));

    for f in fns.iter().take(args.limit) {
        println!("{f}");
    }

    Ok(())
}
