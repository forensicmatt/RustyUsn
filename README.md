# RustyUsn
A fast and cross platform USN Parser writen in Rust.

```
RusyUsn 0.2.0
Matthew Seyer <matthew.seyer@gmail.com>
USN Parser writen in Rust. Check for updates at https://github.com/forensicmatt/RustyUsn

USAGE:
    RusyUsn.exe [FLAGS] --journal <FILE>

FLAGS:
    -h, --help       Prints help information
    -p, --pipe       Input from piped stdin
    -V, --version    Prints version information
    -v, --verbose    Verbose output for debug

OPTIONS:
    -j, --journal <FILE>    The USN journal file to parse
```

## Output
The output is writen to stdout as tab seperated values.

```
target\release\RustyUsn.exe -j testdata\record.usn

offset	record_length	major_version	minor_version	file_reference_number	parent_file_reference_number	usn	timestamp	reason	source_info	security_id	file_attributes	file_name_length	file_name_offset	file_name
0	96	2	0	10477624533077459059	1970324837116475	20342374400	2013-10-19 12:16:53.276040	2	0	0	8224	32	60	BTDevManager.log

```

## Times
Here are some benchmarks ran on a USN Journal file that contains 367260 records and is 35.9 MB (37,687,192 bytes). For this set, both methods yielded the same results.

```
PS E:\RustyUsn\target\release> Measure-Command {.\RustyUsn.exe -j E:\Testing\UsnJrnl.J}

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 1
Milliseconds      : 362
Ticks             : 13627682
TotalDays         : 1.57727800925926E-05
TotalHours        : 0.000378546722222222
TotalMinutes      : 0.0227128033333333
TotalSeconds      : 1.3627682
TotalMilliseconds : 1362.7682
```

```
PS E:\RustyUsn\target\release> Measure-Command {type E:\Testing\UsnJrnl.J | .\RustyUsn.exe -p}

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 2
Milliseconds      : 689
Ticks             : 26892758
TotalDays         : 3.11258773148148E-05
TotalHours        : 0.000747021055555556
TotalMinutes      : 0.0448212633333333
TotalSeconds      : 2.6892758
TotalMilliseconds : 2689.2758
```

## Carving
The idea is to beable to parse records from stdin. You can grab unallocated with the Sleuthkit's blkls. Currently this has failed with RustyUsn.exe dying in some tests. I think more error checks are needed.
```
blkls.exe -o OFFSET IMAGEPATH | RustyUsn.exe -p > carved_records.txt
```

## Build
All you need is a ```cargo build --release``` for compiling with Rust. Currently using Rust 1.15.0 Nightly.

## Change Log
#### RustyUsn 0.3.0 (2017-02-10)
- Added human readable flags by default and option for integer flags (-f --flag)

#### RustyUsn 0.2.1 (2017-02-09)
- Using buffering with the seek_bufread library for better File IO operations.

#### RustyUsn 0.2.0 (2017-02-08)
- Parse from STDIN with -p option
- Added Tests
- Internal Restructure
