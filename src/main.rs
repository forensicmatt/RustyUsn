mod usnpkg;
extern crate clap;
use clap::{App, Arg};
use std::fs::File;

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

    if let Some(journal_name) = options.value_of("journal") {
        println!(
            "Journal to parse: {}",
            journal_name
        );
    }

    // File::open returns a result
    // let usn_fh = match File::open(options.value_of("journal").unwrap()) {
    //     Ok(usn_fh) => usn_fh,
    //    // Handle error here
    //    Err(error) => panic!("Error: {}",error)
    // };
    let mut x = usnpkg::usn::open_file(options.value_of("journal").unwrap());
    x.get_record();
    // let usn_connection = usnpkg::usn::UsnConnection::new(
    //     usn_fh
    // );
    // usn_connection.get_record()

    println!("finish");
}
