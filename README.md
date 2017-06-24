# RustyUsn
A fast and cross platform USN Parser written in Rust that gives you the ability to query the records via JMES Query. Output is JSONL (http://jsonlines.org/).

```
RusyUsn 0.5.0
Matthew Seyer <https://github.com/forensicmatt/RustyUsn>
USN Parser written in Rust.

USAGE:
    RustyUsn.exe [FLAGS] [OPTIONS] --source <PATH>

FLAGS:
    -b, --bool_expr    JMES Query as bool only. (Prints whole record if true.)
    -f, --flags        Print flags as integers and not strings
    -h, --help         Prints help information
    -r, --nonest       Do not use nested references.
    -p, --pipe         Input from piped stdin
    -V, --version      Prints version information
    -v, --verbose      Verbose output for debug

OPTIONS:
    -q, --query <QUERY>    JMES Query
    -s, --source <PATH>    The USN journal file or folder with journals to parse.
```

## Output
The output is written to stdout as a json list of records.

```
// DEFALUT OUTPUT
RustyUsn>target\release\RusyUsn.exe -s testdata\record.usn
{"record_length":96,"major_version":2,"minor_version":0,"file_reference_number":{"reference":"10477624533077459059","entry":115,"sequence":37224},"parent_file_reference_number":{"reference":"1970324837116475","entry":141883,"sequence":7},"usn":20342374400,"timestamp":"2013-10-19 12:16:53.276","reason":"USN_REASON_DATA_EXTEND","source_info":"","security_id":0,"file_attributes":8224,"file_name_length":32,"file_name_offset":60,"file_name":"BTDevManager.log"}

// DO NOT USE NESTED FILE REFERENCES
RustyUsn>target\release\RustyUsn.exe --nonest -s testdata\record.usn
{"record_length":96,"major_version":2,"minor_version":0,"file_reference_number":"10477624533077459059","parent_file_reference_number":"1970324837116475","usn":20342374400,"timestamp":"2013-10-19 12:16:53.276","reason":"USN_REASON_DATA_EXTEND","source_info":"","security_id":0,"file_attributes":8224,"file_name_length":32,"file_name_offset":60,"file_name":"BTDevManager.log"}

// DISPLAY FLAGS AS INTEGERS
RustyUsn>target\release\RustyUsn.exe -f -s testdata\record.usn
{"record_length":96,"major_version":2,"minor_version":0,"file_reference_number":{"reference":"10477624533077459059","entry":115,"sequence":37224},"parent_file_reference_number":{"reference":"1970324837116475","entry":141883,"sequence":7},"usn":20342374400,"timestamp":"2013-10-19 12:16:53.276","reason":2,"source_info":0,"security_id":0,"file_attributes":8224,"file_name_length":32,"file_name_offset":60,"file_name":"BTDevManager.log"}

```

## Query Records
### Reformating using JMES Query
```
// REFORMAT JSON OUTPUT USING A JMES QUERY
RustyUsn>target\release\RustyUsn.exe -s testdata\record.usn -q "[timestamp,file_name,reason]"
["2013-10-19 12:16:53.276","BTDevManager.log","USN_REASON_DATA_EXTEND"]
```
### Filtering using JMES Query
Using the `-b` option will make the query use a bool value to filter results.
```
// FILTER BY AN EXTENTION
RustyUsn>target\release\RustyUsn.exe -s testdata\record.usn -b -q "ends_with(file_name,'.log')"
{"record_length":96,"major_version":2,"minor_version":0,"file_reference_number":{"reference":"10477624533077459059","entry":115,"sequence":37224},"parent_file_reference_number":{"reference":"1970324837116475","entry":141883,"sequence":7},"usn":20342374400,"timestamp":"2013-10-19 12:16:53.276","reason":"USN_REASON_DATA_EXTEND","source_info":"","security_id":0,"file_attributes":8224,"file_name_length":32,"file_name_offset":60,"file_name":"BTDevManager.log"}

// FILTER WHERE RECORD HAS FILE_DELETE FLAG AND NAME ENDS WITH PF (DELETED PREFETCH FILES)
RustyUsn>target\release\RustyUsn.exe -b -q "contains(reason,'USN_REASON_FILE_DELETE')&&ends_with(file_name,'.pf')" -s $UsnJrnl.$J
{"record_length":112,"major_version":2,"minor_version":0,"file_reference_number":{"reference":"1125899906890782","entry":48158,"sequence":4},"parent_file_reference_number":{"reference":"562949953700461","entry":279149,"sequence":2},"usn":20371582824,"timestamp":"2013-10-21 19:46:03.599","reason":"USN_REASON_CLOSE | USN_REASON_FILE_DELETE","source_info":"","security_id":0,"file_attributes":8224,"file_name_length":48,"file_name_offset":60,"file_name":"REGSVR32.EXE-1098A44D.pf"}
{"record_length":112,"major_version":2,"minor_version":0,"file_reference_number":{"reference":"2814749767126627","entry":20067,"sequence":10},"parent_file_reference_number":{"reference":"562949953700461","entry":279149,"sequence":2},"usn":20371582976,"timestamp":"2013-10-21 19:46:03.599","reason":"USN_REASON_CLOSE | USN_REASON_FILE_DELETE","source_info":"","security_id":0,"file_attributes":8224,"file_name_length":46,"file_name_offset":60,"file_name":"DLLHOST.EXE-BCD52255.pf"}
...
```

## Carving
The idea is to be able to parse records from stdin. You can grab unallocated with the Sleuthkit's blkls. Currently this has failed with RustyUsn.exe dying in some tests. I think more error checks are needed.
```
blkls.exe -o OFFSET IMAGEPATH | RustyUsn.exe -p > carved_records.txt
```

## Build
All you need is a ```cargo build --release``` for compiling with Rust. Currently using Rust 1.15.0 Nightly.

## Change Log
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
