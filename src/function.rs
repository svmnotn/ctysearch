use core::fmt::Display;
use std::path::{Path, PathBuf};
use tree_sitter::Point;

use crate::error::Error;

#[derive(Debug)]
pub struct Type<'a> {
    name: &'a str,
    is_pointer: bool,
}

impl<'a> Display for Type<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, if self.is_pointer { "*" } else { "" },)
    }
}

#[derive(Debug)]
pub struct Args<'a>(Vec<Type<'a>>);

impl<'a> Display for Args<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => Ok(()),
            1 => write!(f, "{}", self.0[0]),
            _ => {
                for i in 0..self.0.len() - 2 {
                    write!(f, "{}, ", self.0[i])?;
                }
                write!(f, "{}, ", self.0[self.0.len() - 1])
            }
        }
    }
}

#[derive(Debug)]
pub struct FunctionBuilder<'a> {
    translation_unit: PathBuf,
    pos: Point,
    ret: Option<Type<'a>>,
    name: Option<&'a str>,
    args: Args<'a>,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(translation_unit: &Path, pos: Point) -> Self {
        Self {
            translation_unit: translation_unit.to_path_buf(),
            pos,
            ret: None,
            name: None,
            args: Args(Vec::new()),
        }
    }

    pub fn set_return(&mut self, name: &'a str, is_pointer: bool) {
        self.ret = Some(Type { name, is_pointer });
    }

    pub fn set_name(&mut self, name: &'a str) {
        self.name = Some(name);
    }

    pub fn add_arg(&mut self, name: &'a str, is_pointer: bool) {
        self.args.0.push(Type { name, is_pointer });
    }

    pub fn build(self) -> Result<Function<'a>, Error> {
        Ok(Function {
            translation_unit: self.translation_unit,
            pos: self.pos,
            ret: self.ret.ok_or(Error::NeverFoundAReturnType)?,
            name: self.name.ok_or(Error::NeverFoundAName)?,
            args: self.args,
        })
    }
}

#[derive(Debug)]
pub struct Function<'a> {
    translation_unit: PathBuf,
    pos: Point,
    ret: Type<'a>,
    name: &'a str,
    args: Args<'a>,
}

impl<'a> Function<'a> {
    pub fn canonicalize(&self) -> String {
        format!("{ret} ({args})", ret = self.ret, args = self.args)
    }
}

impl<'a> Display for Function<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{file}:{row}:{col}: {name} :: {ret} ({args})",
            file = self.translation_unit.display(),
            row = self.pos.row + 1,
            col = self.pos.column + 1,
            name = self.name,
            ret = self.ret,
            args = self.args
        )
    }
}
