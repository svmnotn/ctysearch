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
                for i in 0..self.0.len() - 1 {
                    write!(f, "{}, ", self.0[i])?;
                }
                write!(f, "{}", self.0[self.0.len() - 1])
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn point(row: usize, column: usize) -> Point {
        Point { row, column }
    }

    #[test]
    fn type_display_without_pointer() {
        let path = Path::new("test.h");
        let mut builder = FunctionBuilder::new(path, point(0, 0));
        builder.set_return("void", false);
        builder.set_name("f");
        let f = builder.build().unwrap();
        assert_eq!(f.canonicalize(), "void ()");
    }

    #[test]
    fn type_display_with_pointer_return() {
        let path = Path::new("test.h");
        let mut builder = FunctionBuilder::new(path, point(0, 0));
        builder.set_return("uint8_t", true);
        builder.set_name("test");
        builder.add_arg("uint8_t", false);
        let f = builder.build().unwrap();
        assert_eq!(f.canonicalize(), "uint8_t* (uint8_t)");
    }

    #[test]
    fn args_display_zero_one_two_three() {
        let path = Path::new("test.h");

        let mut no_args = FunctionBuilder::new(path, point(0, 0));
        no_args.set_return("void", false);
        no_args.set_name("no_args");
        assert_eq!(no_args.build().unwrap().canonicalize(), "void ()");

        let mut one_arg = FunctionBuilder::new(path, point(0, 0));
        one_arg.set_return("void", false);
        one_arg.set_name("one_arg");
        one_arg.add_arg("void", false);
        assert_eq!(one_arg.build().unwrap().canonicalize(), "void (void)");

        let mut two_args = FunctionBuilder::new(path, point(0, 0));
        two_args.set_return("void", false);
        two_args.set_name("two_args");
        two_args.add_arg("uint8_t", false);
        two_args.add_arg("uint8_t", true);
        assert_eq!(
            two_args.build().unwrap().canonicalize(),
            "void (uint8_t, uint8_t*)"
        );

        let mut three_args = FunctionBuilder::new(path, point(0, 0));
        three_args.set_return("int", false);
        three_args.set_name("three_args");
        three_args.add_arg("int", false);
        three_args.add_arg("char", true);
        three_args.add_arg("void", true);
        assert_eq!(
            three_args.build().unwrap().canonicalize(),
            "int (int, char*, void*)"
        );
    }

    #[test]
    fn function_display_includes_location_name_and_signature() {
        let path = Path::new("example.h");
        let mut builder = FunctionBuilder::new(path, point(4, 0));
        builder.set_return("void", false);
        builder.set_name("test1");
        builder.add_arg("uint8_t", false);
        builder.add_arg("uint8_t", true);
        let f = builder.build().unwrap();
        assert_eq!(
            f.to_string(),
            "example.h:5:1: test1 :: void (uint8_t, uint8_t*)"
        );
    }

    #[test]
    fn builder_fails_without_return_type() {
        let path = Path::new("test.h");
        let mut builder = FunctionBuilder::new(path, point(0, 0));
        builder.set_name("f");
        assert!(matches!(builder.build(), Err(Error::NeverFoundAReturnType)));
    }

    #[test]
    fn builder_fails_without_name() {
        let path = Path::new("test.h");
        let mut builder = FunctionBuilder::new(path, point(0, 0));
        builder.set_return("void", false);
        assert!(matches!(builder.build(), Err(Error::NeverFoundAName)));
    }
}
