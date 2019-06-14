#[macro_use]
extern crate log;
extern crate clap;
extern crate chrono;
use std::thread;
use std::sync::mpsc;
use log::LevelFilter;
use std::process::exit;
use rusty_usn::record::UsnEntry;
use clap::{App, Arg, ArgMatches};
use std::sync::mpsc::{Sender, Receiver};
use rusty_usn::listener::listener::UsnVolumeListener;

static VERSION: &'static str = env!("CARGO_PKG_VERSION");



fn make_app<'a, 'b>() -> App<'a, 'b> {
    let source_arg = Arg::with_name("source")
        .short("s")
        .long("source")
        .value_name("PATH")
        .help("The source volume to listen to. (ie '\\\\.\\C:'")
        .takes_value(true);

    let verbose = Arg::with_name("debug")
        .short("-d")
        .long("debug")
        .value_name("DEBUG")
        .takes_value(true)
        .possible_values(&["Off", "Error", "Warn", "Info", "Debug", "Trace"])
        .help("Debug level to use.");

    App::new("listen_usn")
        .version(VERSION)
        .author("Matthew Seyer <https://github.com/forensicmatt/RustyUsn>")
        .about("USN listener written in Rust. Output is JSONL.")
        .arg(source_arg)
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


fn process_volume(volume_str: &str, _options: &ArgMatches) {
    info!("listening on {}", volume_str);

    let (tx, rx): (Sender<UsnEntry>, Receiver<UsnEntry>) = mpsc::channel();

    let volume_listener = UsnVolumeListener::new(
        volume_str.to_string(),
        tx.clone()
    );

    let _thread = thread::spawn(move || {
        volume_listener.listen_to_volume()
    });

    loop{
        match rx.recv() {
            Ok(entry) => {
                let json_str = serde_json::to_string(&entry).unwrap();
                println!("{}", json_str);
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }
}


fn main() {
    let app = make_app();
    let options = app.get_matches();

    set_debug_level(&options);

    let source_volume = match options.is_present("source") {
        true => {
            match options.value_of("source") {
                Some(path_location) => {
                    path_location
                },
                None => {
                    eprintln!("listen_usn requires a source volume.");
                    exit(-1);
                }
            }
        },
        false => {
            eprintln!("listen_usn requires a source volume.");
            exit(-1);
        }
    };

    process_volume(source_volume, &options);
}
