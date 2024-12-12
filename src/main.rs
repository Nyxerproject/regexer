use clap::{Arg, ArgAction, Command};
use color_eyre::Result;
use std::process;

mod app;
mod custom_regex;
mod engines; // The custom regex module you already have

use app::App;
use engines::{apply_pattern, EngineChoice}; // We'll use apply_pattern inside App as needed

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = Command::new("regexer")
        .version("0.1.0")
        .about("A regex CLI/TUI tool for parsing and testing regular expressions.")
        .long_about(
"regexer is a command-line/text-user interface tool for parsing and testing regular expressions.

...
Use --engine to select the regex engine: (some are wip and are not implemented)
  - builtin
  - custom
  - dfa
  - hybrid
  - onepass
  - boundedbacktracker
  - pikevm 
  - meta
  - custommeta (tries CustomRegex first, verify with builtin, fallback to builtin on error)
"
        )
        .arg(
            Arg::new("pattern")
                .help("The regular expression pattern to match")
                .required(false)
        )
        .arg(
            Arg::new("text")
                .help("The text to search within (use -f to read from a file)")
                .required(false)
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Launch the interactive TUI mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Read text from a file instead of standard input")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Write the output to a file instead of standard output")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("engine")
                .long("engine")
                .help("Select the regex engine to use: builtin, custom, dfa, hybrid, onepass, boundedbacktracker, pikevm, meta, custommeta")
                .value_parser(["builtin", "custom", "dfa", "hybrid", "onepass", "boundedbacktracker", "pikevm", "meta", "custommeta"])
                .default_value("builtin")
        )
        .get_matches();

    let interactive = matches.get_flag("interactive");
    let file = matches.get_one::<String>("file");
    let output = matches.get_one::<String>("output");
    let pattern = matches.get_one::<String>("pattern");
    let text = matches.get_one::<String>("text");
    let engine_str = matches.get_one::<String>("engine").unwrap();
    let engine_choice = engines::parse_engine_choice(engine_str);

    let no_args_provided =
        !interactive && file.is_none() && output.is_none() && pattern.is_none() && text.is_none();
    if no_args_provided {
        eprintln!("No arguments provided. See --help for usage.");
        process::exit(1);
    }

    if !interactive {
        if file.is_some() {
            if pattern.is_none() || text.is_some() {
                eprintln!("When using -f FILE, you must provide PATTERN and must not provide TEXT. See --help for usage.");
                process::exit(1);
            }
        } else {
            if pattern.is_none() || text.is_none() {
                eprintln!("Non-interactive mode requires both PATTERN and TEXT if not using -f FILE. See --help for usage.");
                process::exit(1);
            }
        }
    }

    println!("Running regexer with the following options:");
    if interactive {
        println!("  - Running in interactive mode");
    }
    if let Some(file_name) = file {
        println!("  - Using file input: {}", file_name);
    }
    if let Some(output_file) = output {
        println!("  - Output file: {}", output_file);
    }
    if let Some(p) = pattern {
        println!("  - Pattern: {}", p);
    }
    if let Some(t) = text {
        println!("  - Text: {}", t);
    }
    println!("  - Engine: {}", engine_str);

    if interactive {
        let mut app = App::new(engine_choice);

        if let Some(p) = pattern {
            app.set_pattern(p);
        }
        if let Some(t) = text {
            app.set_text(t);
        }
        if file.is_some() {
            app.set_file(file.map(|f| f.to_string()));
        }

        if app.pattern_is_empty() && app.has_file() {
            app.enter_pattern_mode();
        }

        let terminal = ratatui::init();
        let app_result = app.run(terminal);
        ratatui::restore();
        app_result
    } else {
        // Non-interactive: just run pattern on text/file if needed
        // For now, we do nothing except print out matches if desired.
        Ok(())
    }
}

