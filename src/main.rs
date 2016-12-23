mod usnpkg;
extern crate clap;
use clap::{App, Arg};

fn main() {
    // define the journal parameter
    let journal_arg = Arg::with_name("journal")
        .short("j")
        .long("journal")
        .value_name("FILE")
        .help("The USN journal file to parse")
        .takes_value(true)
        .required(true);

    let options = App::new("MyUsnApp")
        .version("1.0")
        .author("Matthew Seyer <matthew.seyer@gmail.com>")
        .about("Parse USN records")
        .arg(journal_arg) // add the journal parameter
        .get_matches();

    // output journal name
    if let Some(journal_name) = options.value_of("journal") {
        println!(
            "Journal to parse: {}",
            journal_name
        );
    }

    // get UsnConnection from a filename
    let mut usn_connection = usnpkg::usn::open_file(
        options.value_of("journal").unwrap()
    );

    let mut cnt = 1;
    // iterate through each record in the journal
    // We need to add error checking here and make sure
    // we dont have an error other than end of file.
    while let Ok(record) = usn_connection.get_next_record(){
        println!("USN structure {}: {:#?}",cnt,record);
        cnt += 1;
    };
}
