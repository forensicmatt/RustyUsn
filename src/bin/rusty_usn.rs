#[macro_use]
extern crate log;
extern crate clap;
extern crate chrono;
use std::fs;
use std::path::Path;
use log::LevelFilter;
use std::process::exit;
use clap::{App, Arg, ArgMatches};
use rusty_usn::usn::{UsnParserSettings, UsnParser};

static VERSION: &'static str = env!("CARGO_PKG_VERSION");


fn is_a_non_negative_number(value: String) -> Result<(), String> {
    match value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Expected value to be a positive number.".to_owned()),
    }
}


fn make_app<'a, 'b>() -> App<'a, 'b> {
    let source_arg = Arg::with_name("source")
        .short("s")
        .long("source")
        .value_name("PATH")
        .help("The source to parse.")
        .takes_value(true);

    let thread_count = Arg::with_name("threads")
        .short("-t")
        .long("--threads")
        .default_value("0")
        .validator(is_a_non_negative_number)
        .help("Sets the number of worker threads, defaults to number of CPU cores.");

    let verbose = Arg::with_name("debug")
        .short("-d")
        .long("debug")
        .value_name("DEBUG")
        .takes_value(true)
        .possible_values(&["Off", "Error", "Warn", "Info", "Debug", "Trace"])
        .help("Debug level to use.");

    App::new("rusty_usn")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/RustyUsn>")
        .about("USN Parser written in Rust.")
        .arg(source_arg)
        .arg(thread_count)
        .arg(verbose)
}


fn set_debug_level(matches: &ArgMatches){
    // Get the possible logging level supplied by the user
    let message_level = match matches.is_present("debug") {
        true => {
            match matches.value_of("debug") {
                Some("Off") => LevelFilter::Off,
                Some("Error") => LevelFilter::Error,
                Some("Warn") => LevelFilter::Warn,
                Some("Info") => LevelFilter::Info,
                Some("Debug") => LevelFilter::Debug,
                Some("Trace") => LevelFilter::Trace,
                Some(unknown) => {
                    eprintln!("Unknown debug level [{}]", unknown);
                    exit(-1);
                },
                None => {
                    LevelFilter::Off
                }
            }
        },
        false => LevelFilter::Off
    };

    // Create logging with debug level that prints to stderr
    let result = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(message_level)
        .chain(std::io::stderr())
        .apply();
    
    // Ensure that logger was dispatched
    match result {
        Ok(_) => trace!("Logging as been initialized!"),
        Err(error) => {
            eprintln!("Error initializing fern logging: {}", error);
            exit(-1);
        }
    }
}


fn is_directory(source: &str)->bool{
    // Check if a source is a directory
    let metadata = match fs::metadata(source) {
        Ok(meta) => meta,
        Err(error) => {
            eprintln!("{} does not exists. {}", source, error);
            exit(-1);
        }
    };

    let file_type = metadata.file_type();
    file_type.is_dir()
}


fn process_file(file_location: &str, options: &ArgMatches) {
    info!("processing {}", file_location);

    let threads = options
            .value_of("threads")
            .and_then(|value| Some(value.parse::<usize>().expect("used validator")));

    let threads = match (cfg!(feature = "multithreading"), threads) {
        (true, Some(number)) => number,
        (true, None) => 0,
        (false, _) => {
            eprintln!("turned on threads, but library was compiled without `multithreading` feature!");
            1
        }
    };

    let config = UsnParserSettings::new().thread_count(threads);

    let mut parser = match UsnParser::from_path(file_location) {
        Ok(parser) => parser.with_configuration(config),
        Err(error) => {
            eprintln!("Error creating parser for {}: {}", file_location, error);
            return;
        }
    };

    for record in parser.records(){
        let json_str = serde_json::to_string(&record).unwrap();
        println!("{}", json_str);
    }
}


fn main() {
    let app = make_app();
    let options = app.get_matches();

    set_debug_level(&options);

    let source_location = match options.is_present("source") {
        true => {
            match options.value_of("source") {
                Some(path_location) => {
                    // Verify that the supplied path exists
                    if !Path::new(path_location).exists() {
                        eprintln!("{} does not exist.", path_location);
                        exit(-1);
                    }

                    path_location
                },
                None => {
                    eprintln!("usn_dump requires a source to parse.");
                    exit(-1);
                }
            }
        },
        false => {
            eprintln!("usn_dump requires a source to parse.");
            exit(-1);
        }
    };

    if is_directory(source_location) {
        eprintln!("directory as a source is not currently implemented.");
        exit(-1);
    } else {
        process_file(source_location, &options);
    }
}
