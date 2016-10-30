mod usnpkg;
extern crate clap;
use clap::{App, Arg};

fn main() {
    let options = App::new("MyUsnApp")
        .version("1.0")
        .author("Matthew Seyer <matthew.seyer@gmail.com>")
        .about("Parse USN records")
        .arg(Arg::with_name("journal")
            .short("j")
            .long("journal")
            .value_name("FILE")
            .help("The USN journal file to parse")
            .takes_value(true)
            .required(true))
        .get_matches();

    // output journal name
    if let Some(journal_name) = options.value_of("journal") {
        println!(
            "Journal to parse: {}",
            journal_name
        );
    }

    // get usn_connection from a filename
    let mut usn_connection = usnpkg::usn::open_file(
        options.value_of("journal").unwrap()
    );

    // get a record
    let record = usn_connection.get_record();
    // print a record
    println!("USN structure: {:#?}", record);
}
