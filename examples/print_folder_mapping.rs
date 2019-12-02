#[macro_use]
extern crate log;
extern crate clap;
use log::LevelFilter;
use std::process::exit;
use clap::{App, Arg, ArgMatches};
use rusty_usn::mapping::FolderMapping;

static VERSION: &'static str = "0.0.1";


fn make_app<'a, 'b>() -> App<'a, 'b> {
    let source_arg = Arg::with_name("source")
        .short("s")
        .long("source")
        .value_name("PATH")
        .help("The mft file.")
        .takes_value(true);

    let entry_arg = Arg::with_name("entry")
        .long("entry")
        .value_name("ENTRY")
        .help("The entry to lookup.")
        .requires("sequence")
        .takes_value(true);

    let sequence_arg = Arg::with_name("sequence")
        .long("sequence")
        .value_name("SEQUENCE")
        .help("The sequence to lookup.")
        .takes_value(true);

    let verbose = Arg::with_name("debug")
        .short("-d")
        .long("debug")
        .value_name("DEBUG")
        .takes_value(true)
        .possible_values(&["Off", "Error", "Warn", "Info", "Debug", "Trace"])
        .help("Debug level to use.");

    App::new("print_folder_mapping")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/RustyUsn>")
        .about("Print folder mapping from mft.")
        .arg(source_arg)
        .arg(entry_arg)
        .arg(sequence_arg)
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


fn main() {
    let app = make_app();
    let options = app.get_matches();

    set_debug_level(&options);

    let source_mft = match options.is_present("source") {
        true => {
            match options.value_of("source") {
                Some(path_location) => {
                    path_location
                },
                None => {
                    eprintln!("print_folder_mapping requires a source file.");
                    exit(-1);
                }
            }
        },
        false => {
            eprintln!("print_folder_mapping requires a source file.");
            exit(-1);
        }
    };

    let mut mapping = match FolderMapping::from_mft_path(
        source_mft
    ) {
        Ok(mapping) => mapping,
        Err(error) => {
            eprintln!("error creating mapping: {}", error);
            exit(-1);
        }
    };

    if options.is_present("entry") {
        let entry = options.value_of("entry").unwrap().parse::<u64>().unwrap();
        let sequence = options.value_of("sequence").unwrap().parse::<u16>().unwrap();

        let full_path = match mapping.enumerate_path(entry, sequence) {
            Some(path) => path,
            None => {
                eprintln!("No mapping found for {}-{}", entry, sequence);
                exit(-1);
            }
        };
        println!("Full path for {}-{}: {}", entry, sequence, full_path);
    } else {
        println!("{:?}", mapping);
    }
}
