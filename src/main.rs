use clap::Parser as cliParser;

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
* ignore path
* different modes
*   - verbose: print sexp in msg
*   - strict (default) current setup
*/

#[derive(cliParser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// paths to python projects or files
    #[arg(required = true)]
    args: Vec<String>,

    /// Path to ignore
    #[arg(short, long, num_args=1.., value_delimiter = ' ')]
    ignore: Option<Vec<String>>,

    /// Increase output verbosity
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the program options
    let cli = Cli::parse();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("Error loading Python grammar");

    // if there are files, iterate over them
    if let Some(files) = comment_checker::get_modules(cli.args, &cli.ignore) {
        let start = time::Instant::now();
        let mut valid_code: HashMap<String, Vec<String>> = HashMap::new();
        let n_files = files.len();

        // loop through all the files and look for code in the comments
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
                        cli.verbose,
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
        // print info about what file and line the illegal comment is in
        if !valid_code.is_empty() {
            valid_code.iter().for_each(|(file_name, messages)| {
                println!("{}", file_name);
                messages.iter().rev().for_each(|e| println!("{}", e));
                println!();
            });

            println!(
                "Valid code was found in the project. Please update the comment so that it is not valid python or delete it."
            );
        }
        println!(
            "{} files checked in {:?}",
            n_files,
            time::Instant::now() - start
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
