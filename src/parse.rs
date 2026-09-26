use crate::{
    error::Error,
    function::{Function, FunctionBuilder},
};
use std::collections::HashSet;
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator, Tree};

fn parse(text: &[u8]) -> Result<Tree, Error> {
    let mut parser = Parser::new();
    let language = tree_sitter_c::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(Error::FailedToSetParserLanguage)?;
    parser.parse(text, None).ok_or(Error::FailedToParse)
}

fn query() -> Result<Query, Error> {
    const QUERY: &str = include_str!("../query.txt");

    let language = tree_sitter_c::LANGUAGE.into();
    Query::new(&language, QUERY).map_err(Error::FailedToCreateQuery)
}

pub fn find_all<'a>(
    translation_unit: &'a Path,
    text: &'a [u8],
) -> Result<Vec<Function<'a>>, Error> {
    let tree = parse(text)?;
    let query = query()?;
    let mut cursor = QueryCursor::new();
    let mut captured = cursor.matches(&query, tree.root_node(), text.as_ref());

    let parts = query.capture_names();
    let mut fns = Vec::new();
    // The query yields one match per parameter (`@param` occurs inside a
    // `*` repetition), so multi-parameter declarations produce several
    // matches sharing the same `@dcl` node. The first match for a given
    // declaration already collects *all* parameters via the sibling walk
    // below, so skip subsequent matches for the same declaration.
    let mut seen_declarations = HashSet::new();

    while let Some(m) = captured.next() {
        let mut fn_builder = None;
        let mut is_duplicate = false;

        for capture in m.captures() {
            let node = capture.node;
            let index = capture.index;
            match parts[index as usize] {
                "dcl" => {
                    let pos = node.start_position();
                    if !seen_declarations.insert((pos.row, pos.column)) {
                        is_duplicate = true;
                        break;
                    }
                    fn_builder.replace(FunctionBuilder::new(
                        translation_unit,
                        node.start_position(),
                    ));
                }

                "ret" => {
                    fn_builder
                        .as_mut()
                        .ok_or(Error::FoundReturnBeforeDeclaration)?
                        .set_return(
                            node.utf8_text(text.as_ref())
                                .map_err(Error::FailedToReadUtf8)?,
                            node.next_named_sibling()
                                .map(|sibling| sibling.kind() == "pointer_declarator")
                                .unwrap_or(false),
                        );
                }

                "name" => {
                    fn_builder
                        .as_mut()
                        .ok_or(Error::FoundNameBeforeDeclaration)?
                        .set_name(
                            node.utf8_text(text.as_ref())
                                .map_err(Error::FailedToReadUtf8)?,
                        );
                }

                "param" => {
                    let f = fn_builder
                        .as_mut()
                        .ok_or(Error::FoundParameterBeforeDeclaration)?;

                    f.add_arg(
                        node.utf8_text(text.as_ref())
                            .map_err(Error::FailedToReadUtf8)?,
                        node.next_named_sibling()
                            .map(|sibling| sibling.kind() == "pointer_declarator")
                            .unwrap_or(false),
                    );

                    let mut sibling = node
                        .parent()
                        .expect("No parent for param node?")
                        .next_named_sibling();
                    while let Some(node) = sibling {
                        let child = node.named_child(0).expect("Param node has no children?");
                        f.add_arg(
                            child
                                .utf8_text(text.as_ref())
                                .map_err(Error::FailedToReadUtf8)?,
                            child
                                .next_named_sibling()
                                .map(|sibling| sibling.kind() == "pointer_declarator")
                                .unwrap_or(false),
                        );
                        sibling = node.next_named_sibling();
                    }
                }

                _ => unreachable!(
                    "Did someone add a new query pattern and didn't add a matching clause?"
                ),
            }
        }

        if is_duplicate {
            continue;
        }

        let f = fn_builder.expect("no query found").build()?;
        fns.push(f);
    }

    Ok(fns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const EXAMPLE: &[u8] = include_bytes!("../example.h");

    fn find_in_example() -> Vec<Function<'static>> {
        // `example.h` is static bytes, so the borrowed `&str`s can live as
        // long as needed for the test. The translation-unit path is leaked
        // on purpose to satisfy the lifetime without filesystem access.
        let path: &'static Path =
            Box::leak(Box::new(Path::new("example.h").to_path_buf()));
        find_all(path, EXAMPLE).expect("example.h should parse")
    }

    #[test]
    fn finds_all_functions_without_duplicates() {
        let fns = find_in_example();
        // 8 plain declarations + 5 function-pointer typedefs + 3 struct
        // members = 16 unique functions.
        assert_eq!(fns.len(), 16);

        let rendered: Vec<String> = fns.iter().map(ToString::to_string).collect();
        let unique: HashSet<&String> = rendered.iter().collect();
        assert_eq!(unique.len(), fns.len(), "duplicate functions found");
    }

    #[test]
    fn finds_expected_names() {
        let fns = find_in_example();
        let rendered = fns.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
        for name in [
            "test ::", "test1 ::", "test2 ::", "test3 ::", "test4_t ::", "test5_t ::",
            "test6_t ::", "test7_t ::", "test8 ::", "test9 ::", "test10 ::",
            "test11_t ::", "test12 ::", "test13 ::", "test14 ::", "test15 ::",
        ] {
            assert!(rendered.contains(name), "missing {name} in:\n{rendered}");
        }
    }

    #[test]
    fn zero_and_void_params() {
        let fns = find_in_example();
        let by_display = |needle: &str| {
            fns.iter()
                .find(|f| f.to_string().contains(needle))
                .unwrap_or_else(|| panic!("missing {needle}"))
                .canonicalize()
        };
        assert_eq!(by_display("test14 ::"), "void ()");
        assert_eq!(by_display("test15 ::"), "void (void)");
    }

    #[test]
    fn multi_param_function_keeps_all_args() {
        let fns = find_in_example();
        let test1 = fns
            .iter()
            .find(|f| f.to_string().contains("test1 ::"))
            .expect("test1 should be found");
        assert_eq!(test1.canonicalize(), "void (uint8_t, uint8_t*)");
    }

    #[test]
    fn pointer_return_is_preserved() {
        let fns = find_in_example();
        let test = fns
            .iter()
            .find(|f| f.to_string().contains("test ::"))
            .expect("test should be found");
        assert_eq!(test.canonicalize(), "uint8_t* (uint8_t)");
    }

    #[test]
    fn struct_and_enum_returns() {
        let fns = find_in_example();
        let by_display = |needle: &str| {
            fns.iter()
                .find(|f| f.to_string().contains(needle))
                .unwrap_or_else(|| panic!("missing {needle}"))
                .canonicalize()
        };
        assert_eq!(
            by_display("test10 ::"),
            "struct Test_t* (struct Test_t, uint8_t*)"
        );
        assert_eq!(
            by_display("test13 ::"),
            "enum Test_t* (enum Test_t, uint8_t*)"
        );
    }

    #[test]
    fn empty_header_yields_no_functions() {
        let path = Path::new("empty.h");
        let fns = find_all(path, b"#ifndef EMPTY_H\n#define EMPTY_H\n#endif\n").unwrap();
        assert!(fns.is_empty());
    }

    #[test]
    fn single_declaration_is_found_with_position() {
        let path = Path::new("single.h");
        let text = b"int add(int a, int b);\n";
        let fns = find_all(path, text).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].to_string(), "single.h:1:1: add :: int (int, int)");
    }
}
