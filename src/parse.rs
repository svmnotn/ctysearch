use crate::{error::Error, function::Function};
use std::path::PathBuf;
use tree_sitter::{Parser, Query, QueryCapture, QueryCursor, QueryMatch, Tree};

fn parse(text: &impl AsRef<[u8]>) -> Result<Tree, Error> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c::language())
        .map_err(Error::FailedToSetParserLanguage)?;
    parser.parse(text, None).ok_or(Error::FailedToParse)
}

const QUERY: &str = include_str!("../query.txt");

pub fn find_all(translation_unit: PathBuf, text: impl AsRef<[u8]>) -> Result<Vec<Function>, Error> {
    let tree = parse(&text)?;
    let query = Query::new(tree_sitter_c::language(), QUERY).map_err(Error::FailedToCreateQuery)?;
    let mut cursor = QueryCursor::new();
    let captured = cursor.matches(&query, tree.root_node(), text.as_ref());

    let parts = query.capture_names();
    let mut stack = Vec::new();

    for QueryMatch { captures, .. } in captured {
        for QueryCapture { node, index } in captures {
            match parts[*index as usize].as_str() {
                "dcl" => {
                    stack.push(Function {
                        translation_unit: translation_unit.clone(),
                        pos: node.start_position(),
                        ret: String::new(),
                        name: String::new(),
                        args: Vec::new(),
                    });
                }

                "ret" => {
                    let mut f = stack.pop().ok_or(Error::FoundReturnBeforeDeclaration)?;

                    f.ret.push_str(
                        node.utf8_text(text.as_ref())
                            .map_err(Error::FailedToReadUtf8)?,
                    );
                    if let Some(sibling) = node.next_named_sibling() {
                        if sibling.kind() == "pointer_declarator" {
                            f.ret.push('*');
                        }
                    }

                    stack.push(f);
                }

                "name" => {
                    let mut f = stack.pop().ok_or(Error::FoundNameBeforeDeclaration)?;

                    f.name = node
                        .utf8_text(text.as_ref())
                        .map_err(Error::FailedToReadUtf8)?
                        .to_owned();

                    stack.push(f);
                }

                "param" => {
                    let mut f = stack.pop().ok_or(Error::FoundParameterBeforeDeclaration)?;

                    let mut arg = String::from(
                        node.utf8_text(text.as_ref())
                            .map_err(Error::FailedToReadUtf8)?,
                    );
                    if let Some(sibling) = node.next_named_sibling() {
                        if sibling.kind() == "pointer_declarator" {
                            arg.push('*');
                        }
                    }
                    f.args.push(arg);
                    let mut sibling = node.parent().unwrap().next_named_sibling();
                    while let Some(node) = sibling {
                        let child = node.named_child(0).unwrap();
                        let mut arg = String::from(
                            child
                                .utf8_text(text.as_ref())
                                .map_err(Error::FailedToReadUtf8)?,
                        );
                        if let Some(sibling) = child.next_named_sibling() {
                            if sibling.kind() == "pointer_declarator" {
                                arg.push('*');
                            }
                        }
                        f.args.push(arg);
                        sibling = node.next_named_sibling();
                    }

                    stack.push(f);
                }

                _ => unreachable!(
                    "Did someone add a new query pattern and didn't add a matching clause?"
                ),
            }
        }
    }

    Ok(stack)
}
