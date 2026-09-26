use crate::{
    error::Error,
    function::{Function, FunctionBuilder},
};
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

    while let Some(m) = captured.next() {
        let mut fn_builder = None;

        for capture in m.captures() {
            let node = capture.node;
            let index = capture.index;
            match parts[index as usize] {
                "dcl" => {
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

        let f = fn_builder.expect("no query found").build()?;
        fns.push(f);
    }

    Ok(fns)
}
