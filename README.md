[![Build Status](https://dev.azure.com/matthewseyer/matthewseyer/_apis/build/status/forensicmatt.RustyUsn?branchName=master&jobName=Job&configuration=Job%20windows-stable)](https://dev.azure.com/matthewseyer/matthewseyer/_build/latest?definitionId=1&branchName=master)
[![Build Status](https://dev.azure.com/matthewseyer/matthewseyer/_apis/build/status/forensicmatt.RustyUsn?branchName=master&jobName=Job&configuration=Job%20linux-stable)](https://dev.azure.com/matthewseyer/matthewseyer/_build/latest?definitionId=1&branchName=master)
[![Build Status](https://dev.azure.com/matthewseyer/matthewseyer/_apis/build/status/forensicmatt.RustyUsn?branchName=master&jobName=Job&configuration=Job%20mac-stable)](https://dev.azure.com/matthewseyer/matthewseyer/_build/latest?definitionId=1&branchName=master)
# RustyUsn
A fast and cross platform USN Parser written in Rust. Output is [JSONL](http://jsonlines.org/).

This does not currently implement records for usn record version 3 and 4.

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
    -d, --debug <DEBUG>        Debug level to use. [possible values: Off, Error, Warn, Info, Debug, Trace]
    -s, --source <PATH>        The source to parse.
    -t, --threads <threads>    Sets the number of worker threads, defaults to number of CPU cores. [default: 0]
```

## Output
Records are written to stdout as jsonl.

```
{"_offset":3272,"record_length":88,"major_version":2,"minor_version":0,"file_reference":{"entry":254303,"sequence":3},"parent_reference":{"entry":174492,"sequence":2},"usn":1231031496,"timestamp":"2018-07-30 19:52:08.147137","reason":"USN_REASON_CLOSE | USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":38,"file_name_length":24,"file_name_offset":60,"file_name":"DEFAULT.LOG2"}
{"_offset":3184,"record_length":88,"major_version":2,"minor_version":0,"file_reference":{"entry":203911,"sequence":2},"parent_reference":{"entry":174492,"sequence":2},"usn":1231031408,"timestamp":"2018-07-30 19:52:08.147137","reason":"USN_REASON_CLOSE | USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":38,"file_name_length":22,"file_name_offset":60,"file_name":"SYSTEM.LOG1"}
```

# Carve USN from Unallocated
To extract unallocated from an image, use the Sleuthkit's `blkls` with the `-A` option and redirect to a file. Pass that file into rusty_usn.exe.

1. Use TSK to extract out unallocated data.
```
D:\Tools\sleuthkit-4.6.6-win32\bin>mmls D:\Images\CTF_DEFCON_2018\Image3-Desktop\Desktop-Disk0.e01
DOS Partition Table
Offset Sector: 0
Units are in 512-byte sectors

      Slot      Start        End          Length       Description
000:  Meta      0000000000   0000000000   0000000001   Primary Table (#0)
001:  -------   0000000000   0001126399   0001126400   Unallocated
002:  000:000   0001126400   0103904587   0102778188   NTFS / exFAT (0x07)
003:  -------   0103904588   0103905279   0000000692   Unallocated
004:  000:001   0103905280   0104855551   0000950272   Unknown Type (0x27)
005:  -------   0104855552   0104857599   0000002048   Unallocated

D:\Tools\sleuthkit-4.6.6-win32\bin>blkls -A -o 1126400 D:\Images\CTF_DEFCON_2018\Image3-Desktop\Desktop-Disk0.e01 > D:\Images\CTF_DEFCON_2018\Image3-Desktop\Desktop-Disk0.unallocated
```

2. Parse the unallocated extracted file with rust_usn.exe.
```
D:\Tools\RustyTools>rusty_usn.exe -s D:\Images\CTF_DEFCON_2018\Image3-Desktop\Desktop-Disk0.unallocated > D:\Testing\unallocated-usn.jsonl
```

3. Count records recovered.
```
D:\Tools\RustyTools>rg -U -c "" D:\Testing\unallocated-usn.jsonl
1558102
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
