# RustyUsn
A fast and cross platform USN Parser written in Rust.

```
RusyUsn 0.4.0
Matthew Seyer <matthew.seyer@gmail.com>
USN Parser written in Rust. Check for updates at https://github.com/forensicmatt/RustyUsn

USAGE:
    RusyUsn.exe [FLAGS] --journal <FILE>

FLAGS:
    -f, --flags      Print flags as integers and not strings
    -h, --help       Prints help information
    -p, --pipe       Input from piped stdin
    -V, --version    Prints version information
    -v, --verbose    Verbose output for debug

OPTIONS:
    -j, --journal <FILE>    The USN journal file to parse
```

## Output
The output is written to stdout as a json list of records.

```
RustyUsn>target\release\RusyUsn.exe -j testdata\record.usn
[
  {
    "record_length": 96,
    "major_version": 2,
    "minor_version": 0,
    "file_reference_number": 10477624533077459059,
    "parent_file_reference_number": 1970324837116475,
    "usn": 20342374400,
    "timestamp": "2013-10-19 12:16:53.276040",
    "reason": "USN_REASON_DATA_EXTEND",
    "source_info": "",
    "security_id": 0,
    "file_attributes": 8224,
    "file_name_length": 32,
    "file_name_offset": 60,
    "file_name": "BTDevManager.log"
  }
]

RustyUsn>target\release\RusyUsn.exe -f -j testdata\record.usn
[
  {
    "record_length": 96,
    "major_version": 2,
    "minor_version": 0,
    "file_reference_number": 10477624533077459059,
    "parent_file_reference_number": 1970324837116475,
    "usn": 20342374400,
    "timestamp": "2013-10-19 12:16:53.276040",
    "reason": 2,
    "source_info": 0,
    "security_id": 0,
    "file_attributes": 8224,
    "file_name_length": 32,
    "file_name_offset": 60,
    "file_name": "BTDevManager.log"
  }
]

```
## Times
Here are some benchmarks ran on a USN Journal file that contains 367260 records and is 35.9 MB (37,687,192 bytes). For this set, both methods yielded the same results.

I have focused on JSON output due to the interest of inserting into NoSQL or indexing. However, JSON serialization is much slower than just printing CSV values. I am posting current benchmarks for the new times. I will keep the old benchmarks for comparison under 'Old CSV Times' section.

```
PS E:\RustyUsn\target\release> Measure-Command {.\RusyUsn.exe -j E:\Testing\UsnJrnl.J}


Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 14
Milliseconds      : 857
Ticks             : 148573743
TotalDays         : 0.000171960350694444
TotalHours        : 0.00412704841666667
TotalMinutes      : 0.247622905
TotalSeconds      : 14.8573743
TotalMilliseconds : 14857.3743
```

```
PS E:\RustyUsn\target\release> Measure-Command {type E:\Testing\UsnJrnl.J | .\RusyUsn.exe -p}


Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 15
Milliseconds      : 313
Ticks             : 153139914
TotalDays         : 0.000177245270833333
TotalHours        : 0.0042538865
TotalMinutes      : 0.25523319
TotalSeconds      : 15.3139914
TotalMilliseconds : 15313.9914
```

## Old CSV Times
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
The idea is to be able to parse records from stdin. You can grab unallocated with the Sleuthkit's blkls. Currently this has failed with RustyUsn.exe dying in some tests. I think more error checks are needed.
```
blkls.exe -o OFFSET IMAGEPATH | RustyUsn.exe -p > carved_records.txt
```

## Build
All you need is a ```cargo build --release``` for compiling with Rust. Currently using Rust 1.15.0 Nightly.

## Change Log
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
