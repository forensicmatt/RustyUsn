#[macro_use] extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate rustyusn;
extern crate serde_json;
extern crate serde;
use usnpkg::flags::{FLAGS_AS_INT};
use serde::Serializer;
use serde::ser::SerializeSeq;
use rustyusn::usnpkg;
use clap::{App, Arg};
use regex::bytes;
use std::io::prelude::*;
use std::io;

const BUFFER_SIZE: usize = 512;
const OVERFLOW_SIZE: usize = 512;

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.join("")
}

pub fn fill_buffer<R: Read>(input_handle: &mut R, buffer_size: usize)->Result<Vec<u8>,io::Error>{
    let mut bytes_in_buf: usize = 0;
    let mut buffer = vec![0; buffer_size];
    let end_ofs: usize = buffer_size;
    let mut start_ofs: usize = 0;
    let mut continue_flag: bool = false;

    loop {
        let bytes_read = match input_handle.read(&mut buffer[start_ofs .. end_ofs]) {
            Ok(bytes_read) => bytes_read,
            Err(error) => {
                return Err(error);
            }
        };

        // add the bytes read to the bytes in buffer count
        bytes_in_buf += bytes_read;

        // if bytes_read is zero, no more input
        if bytes_read == 0 {
            // Only break if there is no continue flag
            if continue_flag == false {
                break;
            }
        }
        // If bytes_in_buf is less than our allocated buffer size lets read more.
        else if bytes_in_buf < BUFFER_SIZE {
            // we need to append our next read to where this buffer ends
            start_ofs = bytes_in_buf;

            // Set continue flag if not set
            if continue_flag == false {
                continue_flag = true;
                continue;
            }
        }

        break;
    }

    buffer.truncate(bytes_in_buf);
    Ok(buffer)
}

fn main() {
    // define the journal parameter
    let journal_arg = Arg::with_name("journal")
        .short("j")
        .long("journal")
        .value_name("FILE")
        .help("The USN journal file to parse")
        .required_unless("pipe")
        .takes_value(true);

    let pipe_arg = Arg::with_name("pipe")
        .short("p")
        .long("pipe")
        .help("Input from piped stdin");

    let flags_arg = Arg::with_name("flags")
        .short("f")
        .long("flags")
        .help("Print flags as integers and not strings");

    let verbose = Arg::with_name("verbose")
        .short("v")
        .long("verbose")
        .help("Verbose output for debug");

    let options = App::new("RusyUsn")
        .version("0.3.0")
        .author("Matthew Seyer <matthew.seyer@gmail.com>")
        .about("USN Parser writen in Rust. Check for updates at https://github.com/forensicmatt/RustyUsn")
        .arg(journal_arg)   // add the journal parameter
        .arg(pipe_arg)      // add the pipe parameter
        .arg(flags_arg)      // add the flags parameter
        .arg(verbose)       // add the verbose parameter
        .get_matches();

    let pipe_flag = options.occurrences_of("pipe");
    let verbose_flag = options.is_present("verbose");
    let int_flags_flag = options.is_present("flags");

    if int_flags_flag {
        unsafe {
            FLAGS_AS_INT = true;
        }
    }

    let mut serializer = serde_json::Serializer::pretty(
        io::stdout()
    );
    let mut seq = serializer.serialize_seq(None).unwrap();

    if pipe_flag == 1 {
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let mut read_buf_flag: bool = true;
        let mut buffer: Vec<u8>;
        let mut overflow: Vec<u8>;
        let mut last_flag: bool = false;

        // initialize buffers
        buffer = vec![0; 0];
        overflow = vec![0; 0];

        lazy_static! {
            static ref RE: bytes::Regex = bytes::Regex::new(
                "..\x00\x00(\x02)\x00\x00\x00"
            ).unwrap();
        }

        let mut total_read = 0;
        // Iterate stdin
        loop {
            if read_buf_flag {
                // Read stdin to buffer
                buffer = match fill_buffer(&mut stdin, BUFFER_SIZE){
                    Ok(buffer) => buffer,
                    Err(error) => panic!(error)
                };

                // break if buffer size is zero
                if buffer.len() == 0 {
                    break;
                }
            }

            // Get overflow for records exceding buffer
            if !last_flag {
                // get overflow buffer
                overflow = match fill_buffer(&mut stdin, OVERFLOW_SIZE){
                    Ok(overflow) => overflow,
                    Err(error) => panic!(error)
                };

                // If there is no overflow we have reached the end of the file
                if overflow.len() == 0 {
                    // Set last_flag so we know to terminate after parsing the current buffer
                    last_flag = true;
                }
            }

            // Create the search buffer (buffer + overflow)
            // We must add overflow because there is posibility that a record could span
            // the buffer. We need to always have at least one mas record length in the
            // overflow.
            let mut search_buffer: Vec<u8> = Vec::new();
            // Add buffer to search_buffer
            search_buffer.extend_from_slice(&buffer[..]);
            if !last_flag {
                // Add overflow to buffer as long as this is not our last search
                search_buffer.extend_from_slice(&overflow[..]);
            }

            // Pick end offset for regex search
            // search only the buffer + what it takes to find a signature
            let mut search_end: usize = buffer.len() + 8;
            if search_buffer.len() < search_end {
                search_end = search_buffer.len();
            }

            // regex buffer
            let mut last_hit_end_ofs: usize = 0;
            for hit in RE.find_iter(&search_buffer[.. search_end]) {
                // set relative location
                let location_rel: usize = hit.0;

                // println!("Hit at: {}",location_rel);

                // attempt to read record from start of match offset
                let record: usnpkg::usn::UsnRecordV2 = match usnpkg::usn::read_record(&search_buffer[location_rel ..]) {
                    Ok(record) => record,
                    Err(error) => {
                        if verbose_flag {
                            println!("{:?}",error);
                        }
                        continue;
                    }
                };

                // set end_ofs
                last_hit_end_ofs = location_rel + (record.record_length as usize);

                // println!("record offsets: {}-{}",total_read + location_rel,total_read + location_rel + (record.record_length as usize));

                seq.serialize_element(&record).unwrap();
            }

            // Check if the end of the last hit was more than the buffer
            let mut next_start: usize = 0;
            if last_hit_end_ofs > buffer.len() {
                // Because the record ran into the overflow we need to
                // set the start point of overflow to copy into the buffer
                // otherwise we can have duplicate hits because we would
                // be parsing already searched buffer
                next_start = last_hit_end_ofs - buffer.len();
            } else {
                // No overlap
                next_start = 0;
            }

            // account to our total read count
            total_read += buffer.len();

            // if this is the last flag we can break
            if last_flag {
                break
            } else {
                // Acount for overlapped data searched for our total_read
                total_read += next_start;
                // copy overflow to our buffer
                buffer = overflow[next_start..].to_vec();
                // because we have filled the buffer from overflow, we do
                // not need to read to our buffer.
                read_buf_flag = false;
            }
        }
    } else {
        // get UsnConnection from a filename
        let mut usn_connection = usnpkg::usn::open_file(
            options.value_of("journal").unwrap(),
            verbose_flag
        );

        // iterate through each record in the journal
        // We need to add error checking here and make sure
        // we dont have an error other than end of file.
        while let Ok(usn_result) = usn_connection.get_next_record(){
            seq.serialize_element(&usn_result.0).unwrap();
        };
    }

    seq.end().unwrap();
}
