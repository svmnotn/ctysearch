use core::fmt::Display;
use std::path::PathBuf;
use tree_sitter::Point;

#[derive(Debug)]
pub struct Function {
    pub translation_unit: PathBuf,
    pub pos: Point,
    pub ret: String,
    pub name: String,
    pub args: Vec<String>,
}

impl Function {
    pub fn cannonize(&self) -> String {
        format!("{} ({})", self.ret, self.args.join(", "))
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} :: {} ({})",
            self.translation_unit.display(),
            self.pos.row + 1,
            self.pos.column + 1,
            self.name,
            self.ret,
            self.args.join(", ")
        )
    }
}
