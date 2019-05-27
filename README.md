# RustyUsn
A fast and cross platform USN Parser written in Rust. Output is [JSONL](http://jsonlines.org/).

```
rusty_usn 1.0.0
Matthew Seyer <https://github.com/forensicmatt/RustyUsn>
USN Parser written in Rust.

USAGE:
    rusty_usn.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --debug <DEBUG>    Debug level to use. [possible values: Off, Error, Warn, Info, Debug, Trace]
    -s, --source <PATH>    The source to parse.
```

## Output
Records are written to stdout as jsonl.

```
{"_offset":3272,"record_length":88,"major_version":2,"minor_version":0,"file_reference":{"entry":254303,"sequence":3},"parent_reference":{"entry":174492,"sequence":2},"usn":1231031496,"timestamp":"2018-07-30 19:52:08.147137","reason":"USN_REASON_CLOSE | USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":38,"file_name_length":24,"file_name_offset":60,"file_name":"DEFAULT.LOG2"}
{"_offset":3184,"record_length":88,"major_version":2,"minor_version":0,"file_reference":{"entry":203911,"sequence":2},"parent_reference":{"entry":174492,"sequence":2},"usn":1231031408,"timestamp":"2018-07-30 19:52:08.147137","reason":"USN_REASON_CLOSE | USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":38,"file_name_length":22,"file_name_offset":60,"file_name":"SYSTEM.LOG1"}
```

## Build
All you need is a ```cargo build --release``` for compiling with Rust. Currently using Rust 1.36.0 Nightly.

## Change Log
#### RustyUsn 1.0.0 (2019-05-27)
- Rewrite and removal of features

#### RustyUsn 0.5.0 (2017-06-22)
- Added JMES Query functionality (http://jmespath.org/)
- Added JSONL output (http://jsonlines.org/)

#### RustyUsn 0.4.1 (2017-04-05)
- Now using r-winstructs for file references and timestamps. This means greater controll of serialization. references are now strings on json serialization.

#### RustyUsn 0.4.0 (2017-03-13)
- Removed CSV output, Added JSON output

#### RustyUsn 0.3.0 (2017-02-10)
- Added human readable flags by default and option for integer flags (-f --flag)

#### RustyUsn 0.2.1 (2017-02-09)
- Using buffering with the seek_bufread library for better File IO operations.

#### RustyUsn 0.2.0 (2017-02-08)
- Parse from STDIN with -p option
- Added Tests
- Internal Restructure
