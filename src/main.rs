use std::env;
use std::fs;
use std::error::Error;
use walkdir::WalkDir;
use glob;
use colored::*;

const INVALID_ARGS_INFO: &str = "Invalid arguments! User -h or --help for usage information.";

const USAGE_INFO: &str =
"Usage: grep [OPTIONS] <pattern> <files...>\n\
Options:\n\
-i                Case-insensitive search\n\
-n                Print line numbers\n\
-v                Invert match (exclude lines that match the pattern)\n\
-r                Recursive directory search\n\
-f                Print filenames\n\
-c                Enable colored output\n\
-h, --help        Show help information";

pub struct Config {
    print_usage: bool,
    search_string: String,
    filenames: Vec<String>,
    is_case_insensitive: bool,
    print_line_no: bool,
    invert_match: bool,
    recursive_search: bool,
    print_filenames: bool,
    coloured_output: bool,
}

impl Config {
    // Parse command line argument and create a Config object
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err(&INVALID_ARGS_INFO);
        }
        
        let mut queries = Vec::<String>::new();
        let mut case_insensitive = false;
        let mut print_line_no = false;
        let mut invert_match = false;
        let mut recursive_search = false;
        let mut print_filenames = false;
        let mut coloured_output = false;
        let mut print_usage = false;
        
        for arg in args.iter() {
            match arg.as_str() {
                "-i" => case_insensitive = true,
                "-n" => print_line_no = true,
                "-v" => invert_match = true,
                "-r" => recursive_search = true,
                "-f" => print_filenames = true,
                "-c" => coloured_output = true,
                "-h" | "--help" => print_usage = true,
                _ => queries.push(arg.clone()),
            }
        }
        
        let mut filenames = Vec::new();
        let mut search_string = String::new();

        if !print_usage && queries.len() < 3 {
            return Err(INVALID_ARGS_INFO);
        } else if !print_usage {
            filenames = queries[2..].to_vec();
            search_string = queries[1].clone();
            
        }
        

        Ok(Config {
            print_usage,
            search_string,
            filenames,
            is_case_insensitive: case_insensitive,
            print_line_no,
            invert_match,
            recursive_search,
            print_filenames,
            coloured_output,
        })
    }
}

fn parse_filenames(filenames: &[String], recursive_search: bool) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::<String>::new();
    for filename in filenames {
        let metadata = fs::metadata(filename)?;
        if metadata.is_dir() {
            if recursive_search {
                for entry in WalkDir::new(filename).into_iter().filter_map(Result::ok) {
                    let path = entry.path();
    
                    if path.is_file() {
                        files.push(path.to_str().unwrap().to_string());
                    }
                }
            } else {
                eprintln!("{} is a directory. Use -r option to search recursively.", filename);
            }
        } else {
            // Check if there is a wildcard in the filename
            if filename.contains('*') {
                let paths = glob::glob(filename)?;
                for path in paths {
                    files.push(path?.to_str().unwrap().to_string());
                }
            } else {
                // Check if file exists
                files.push(filename.clone());
            }
        }
    }
    Ok(files)
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.print_usage {
        println!("{}", &USAGE_INFO);
        return Ok(());
    }

    // Get the files to search (assuming inputs are always valid)
    let files = parse_filenames(&config.filenames, config.recursive_search)?;

    // Open the files
    for file in files {
        let contents = fs::read_to_string(&file)?;
        let lines = contents.lines();
        let mut line_no = 1;

        for line in lines {
            let mut matched: bool;
            if config.is_case_insensitive {
                matched = line.to_lowercase().contains(&config.search_string.to_lowercase());
            } else {
                matched = line.contains(&config.search_string);
            }

            if config.invert_match {
                matched = !matched;
            }

            if matched {
                // Build the output string
                let mut output = String::new();
                if config.print_filenames {
                    output.push_str(&file);
                    output.push_str(": ");
                }
                if config.print_line_no {
                    output.push_str(&line_no.to_string());
                    output.push_str(": ");
                }
                if config.coloured_output && !config.invert_match && !config.is_case_insensitive {
                    // Find the index of the search string in the line, assuming `-i` and `-v` is not defined
                    let index = line.find(&config.search_string).unwrap();
                    print!("{}{}", output, line[0..index].to_string());
                    print!("{}", &line[index..index + config.search_string.len()].red());
                    println!("{}", &line[index + config.search_string.len()..]);
                }
                 else {
                    output.push_str(&line);
                    println!("{}", output);
                }
            }

            line_no += 1;
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config: Config = Config::new(&args).expect(&INVALID_ARGS_INFO);

    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
