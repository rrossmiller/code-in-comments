use std::fs;

use tree_sitter::{Node, Parser};

pub fn get_modules(path: Vec<String>, ignore: &Option<Vec<String>>) -> Option<Vec<String>> {
    let mut rtn: Vec<String> = Vec::new();
    for p in path.iter() {
        if let Ok(exist) = fs::exists(&p) {
            if exist {
                let mut dirs = Vec::new();
                let met = fs::metadata(&p).unwrap();

                // if dir return py contents
                if met.is_dir() {
                    if let Ok(dir) = fs::read_dir(&p) {
                        // loop through dir contents
                        // py files stay in list, dirs are moved to dirs vector for later expansion
                        let mut dir_contents: Vec<String> = dir
                            .filter_map(|e| {
                                if let Ok(f) = e {
                                    if let Some(ext) = f.path().extension() {
                                        if ext == "py" {
                                            return Some(String::from(f.path().to_str().unwrap()));
                                        }
                                    }
                                    // if "f" is a dir, read it's contents later
                                    else if f.path().is_dir() {
                                        dirs.push(String::from(f.path().to_str().unwrap()));
                                    }
                                }
                                None
                            })
                            .collect();
                        rtn.append(&mut dir_contents);

                        // if there are dirs in the path, read their contents
                        // recursively call this func to get contents
                        // TODO
                        if dirs.len() > 0 {
                            if let Some(mods) = get_modules(dirs, &ignore) {
                                let mut mods = mods;
                                rtn.append(&mut mods);
                            }
                        }
                    }

                // if file, return vec of that file name
                } else {
                    rtn.push(p.to_string());
                }
            } else {
                eprintln!("{} does not exist", p);
                return None;
            }
        }
    }

    // filter out ignored paths
    if let Some(i) = ignore {
        rtn = rtn
            .iter()
            .filter_map(|e| {
                if i.contains(e) {
                    return None;
                }
                Some(e.to_string())
            })
            .collect();
    }

    Some(rtn)
}

pub fn comment_has_valid_code(
    comment_node: Node,
    parser: &mut Parser,
    source_code: &String,
    verbose: bool,
) -> Option<String> {
    let br = comment_node.byte_range();
    let comment = &source_code.as_str()[br.start..br.end]
        .chars()
        .collect::<String>();

    // if there's text after the comment start
    if let Some(comment_body) = comment.strip_prefix("#") {
        let comment_tree = parser.parse(comment_body, None).unwrap();
        let comment_root = comment_tree.root_node();

        if !comment_root.has_error() && is_not_allowable_comment(comment_root) {
            let st = comment_node.start_position();
            let st_row = st.row + 1;

            if verbose {
                return Some(format!(
                    "({}, {}): {}\n  \"{}\"",
                    st_row,
                    st.column,
                    comment_root.to_sexp(),
                    comment_body.trim(),
                ));
            }
            return Some(format!(
                "({}, {}):\n  \"{}\"",
                st_row,
                st.column,
                comment_body.trim(),
            ));
        }
    }
    None
}

/// Allow some comments that are valid python
/// i.e.
///     single word comments that could be mistaken for identifier expr
///     singular ints, floats,
///     single keywords
fn is_not_allowable_comment(node: Node) -> bool {
    // check if it's a one word/number comment
    match node.to_sexp().as_str() {
        "(module (comment))"
        | "(module (expression_statement (float)))"
        | "(module (expression_statement (identifier)))"
        | "(module (expression_statement (keyword_identifier)))"
        | "(module (expression_statement (integer)))"
        // | "(module (expression_statement (string (string_start) (string_content) (string_end))))" 

        // things like "pyright: ignore"
        |"(module (expression_statement (assignment left: (identifier) type: (type (identifier)))))" 

        => false,

        // most anything else is illegal
        _ => true,
    }
}
