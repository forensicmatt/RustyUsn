#[macro_use]
extern crate log;
extern crate clap;
extern crate chrono;
use std::fs;
use std::path::Path;
use log::LevelFilter;
use std::process::exit;
use serde_json::value::Value;
use clap::{App, Arg, ArgMatches};
use rusty_usn::mapping::FolderMapping;
use rusty_usn::usn::{UsnParserSettings, UsnParser};
use rusty_usn::record::UsnEntry;
use rusty_usn::record::UsnRecord;
use rusty_usn::flags;

static VERSION: &'static str = "1.2.0";


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
        .help("The source to parse. If the source is a directory, the directoy will \
        be recursed looking for any files that end with '$J'. (Do not use a directory \
        if using an MFT file.)")
        .takes_value(true);

    let usn_arg = Arg::with_name("mft")
        .short("m")
        .long("mft")
        .value_name("MFT")
        .help("The MFT to use for creating folder mapping.")
        .takes_value(true);

    let thread_count = Arg::with_name("threads")
        .short("-t")
        .long("--threads")
        .default_value("0")
        .validator(is_a_non_negative_number)
        .help("Sets the number of worker threads, defaults to number of CPU cores. \
        If the --mft option is used, the tool can only run single threaded.");

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
        .about("USN Parser written in Rust. Output is JSONL.")
        .arg(source_arg)
        .arg(usn_arg)
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


fn process_directory(directory: &str, options: &ArgMatches) {
    for dir_reader in fs::read_dir(directory) {
        for entry_result in dir_reader {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        let path_string = path.into_os_string().into_string().unwrap();
                        if path_string.to_lowercase().ends_with("$j"){
                            process_file(&path_string, &options);
                        }
                    } else if path.is_dir(){
                        let path_string = path.into_os_string().into_string().unwrap();
                        process_directory(&path_string, &options);
                    }
                },
                Err(error) => {
                    eprintln!("Error reading {} [{:?}]", directory, error);
                }
            }
        }
    }
}


fn process_file(file_location: &str, options: &ArgMatches) {
    info!("processing {}", file_location);

    let thread_option = options
            .value_of("threads")
            .and_then(|value| Some(value.parse::<usize>().expect("used validator")));

    let mut threads = match (cfg!(feature = "multithreading"), thread_option) {
        (true, Some(number)) => number,
        (true, None) => 0,
        (false, _) => {
            eprintln!("turned on threads, but library was compiled without `multithreading` feature!");
            1
        }
    };

    let mut folder_mapping: Option<FolderMapping> = None;

    if options.is_present("mft") {
        if threads != 1 {
            threads = 1;
            eprintln!("When using MFT to create folder map, threads can only be 1.");
        }

        let mft_path = options.value_of("mft").unwrap();
        folder_mapping = match FolderMapping::from_mft_path(mft_path){
            Ok(mapping) => Some(mapping),
            Err(err) => {
                eprintln!("Error creating folder mapping. {}", err);
                exit(-1);
            }
        };
    }

    let config = UsnParserSettings::new().thread_count(threads);

    let mut parser = match UsnParser::from_path(file_location) {
        Ok(parser) => parser.with_configuration(config),
        Err(error) => {
            eprintln!("Error creating parser for {}: {}", file_location, error);
            return;
        }
    };

    if folder_mapping.is_some(){
        // Because we are going to enumerate folder names, we must
        // iterate records from the newest to oldest inorder to correctly
        // enumerate the paths. This means we must store all the records 
        // because they are parsed from oldest to newest. Unfortunately,
        // this does take up more memory.
        let mut mapping = folder_mapping.unwrap();
        let mut entry_list: Vec::<UsnEntry> = Vec::new();
        for record in parser.records(){
            entry_list.push(record);
        }
        entry_list.reverse();

        for entry in entry_list {
            let mut entry_json_value = serde_json::to_value(&entry).unwrap();
            let json_map = entry_json_value.as_object_mut().unwrap();
            match entry.record {
                UsnRecord::V2(record) => {
                    if record.file_attributes.contains(flags::FileAttributes::FILE_ATTRIBUTE_DIRECTORY){
                        // Add mapping on a delete or rename old
                        if record.reason.contains(flags::Reason::USN_REASON_FILE_DELETE) ||
                            record.reason.contains(flags::Reason::USN_REASON_RENAME_OLD_NAME) {
                            mapping.add_mapping(
                                record.file_reference,
                                record.file_name.clone(),
                                record.parent_reference
                            );
                        }
                    }

                    // Enumerate the path of this record from the FolderMapping
                    let full_path = match mapping.enumerate_path(
                        record.parent_reference.entry,
                        record.parent_reference.sequence
                    ){
                        Some(path) => path,
                        None => "[Unknown]".to_string()
                    };

                    // Create teh fullname string
                    let full_name = format!("{}/{}", full_path, record.file_name);
                    // Add the fullname string to the json record
                    let fn_value = Value::String(full_name);
                    json_map.insert("full_name".to_string(), fn_value);

                    // Create a json string to print
                    let json_str = serde_json::to_string(&json_map).unwrap();
                    println!("{}", json_str);
                }
            }
        }
    } else{
        for record in parser.records(){
            let json_str = serde_json::to_string(&record).unwrap();
            println!("{}", json_str);
        }
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
        process_directory(source_location, &options);
    } else {
        process_file(source_location, &options);
    }
}
