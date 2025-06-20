use std::{
    collections::{HashMap, VecDeque},
    env::{self},
    fs, io,
    process::exit,
    time,
};

use comment_checker;
use tree_sitter::{Node, Parser};
use tree_sitter_python;
/*
* TODO
* check if removing a comment from the end of a line makes is valid code
*/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("Error loading Python grammar");

    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();
    let args = Vec::from(args);

    if args.len() < 1 {
        eprintln!("usage: ./exe <path to src>");
        exit(1);
    }

    // if there are files, iterate over them
    if let Some(files) = comment_checker::get_modules(args) {
        let start = time::Instant::now();
        let mut valid_code: HashMap<String, Vec<String>> = HashMap::new();
        // filter out directories
        let files: Vec<String> = files
            .iter()
            .filter_map(|p| {
                if let Ok(m) = fs::metadata(p) {
                    if m.is_file() {
                        return Some(p.clone());
                    }
                }
                None
            })
            .collect();
        let n_files = files.len();
        for f in files {
            let source_code =
                read_source_file(&f).expect(format!("Issue reading {}\n", f).as_str());
            let tree = parser.parse(&source_code, None).unwrap();
            let root_node = tree.root_node();
            let mut cursor = tree.walk();

            // dfs until you find a comment
            let mut stack: Vec<Node> = root_node.children(&mut cursor).collect();
            while let Some(node) = stack.pop() {
                // when you find a comment, see if it has valid code.
                if node.grammar_name() == "comment" {
                    if let Some(msg) = comment_checker::comment_has_valid_code(
                        node,
                        &mut parser,
                        &source_code,
                        true,
                    ) {
                        if let Some(v) = valid_code.get_mut(&f) {
                            v.push(msg);
                        } else {
                            valid_code.insert(f.clone(), vec![msg]);
                        }
                    }
                // If the node isn't a comment, continue looking for comments in its children
                } else {
                    stack.append(&mut node.children(&mut cursor).collect());
                }
            }
        }
        // if valid code in comments was found
        // print an error message and exit
        if !valid_code.is_empty() {
            valid_code.iter().for_each(|(k, v)| {
                println!("{}", k);
                v.iter().rev().for_each(|e| println!("{}", e));
                println!();
            });

            println!(
                // "Valid code was found in the project. Please delete it or update the comment so that it is not valid python."
                "Valid code was found in the project. Please update the comment so that it is not valid python or delete it."
            );
        }
        println!(
            "{} files checked in {:?}",
            n_files,
            start.duration_since(time::Instant::now())
        );
        if !valid_code.is_empty() {
            exit(1);
        }

        println!("OK. No comments with valid code found.");
    } else {
        println!("File(s) not found");
    }

    Ok(())
}

// TODO return a buffer instead of the whole file in memory?
fn read_source_file(pth: &String) -> io::Result<String> {
    fs::read_to_string(pth)
}
// fn read_source_file() -> io::Result<BufReader<File>> {
//     let f = File::open("../ex/rate_limiter.py")?;
//     let x = BufReader::new(f);
//     Ok(x)

// let mut line = String::new();
// let len = reader.read_line(&mut line)?;
// println!("First line is {len} bytes long");
// Ok(String::new())
// }
