[![Build Status](https://dev.azure.com/matthewseyer/dfir/_apis/build/status/forensicmatt.RustyUsn?branchName=master)](https://dev.azure.com/matthewseyer/dfir/_build/latest?definitionId=1&branchName=master)
# RustyUsn
A fast and cross platform USN Parser written in Rust. Output is [JSONL](http://jsonlines.org/).

This does not currently implement records for usn record version 3 and 4.

```
rusty_usn 1.1.0
Matthew Seyer <https://github.com/forensicmatt/RustyUsn>
USN Parser written in Rust. Output is JSONL.

USAGE:
    rusty_usn.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --debug <DEBUG>        Debug level to use. [possible values: Off, Error, Warn, Info, Debug, Trace]
    -s, --source <PATH>        The source to parse. If the source is a directory, the directoy will be recursed looking
                               for any files that end with '$J'.
    -t, --threads <threads>    Sets the number of worker threads, defaults to number of CPU cores. [default: 0]
```

## Output
Records are written to stdout as jsonl.

```
{"_source":"D:\\Images\\CTF_DEFCON_2018\\Image3-Desktop\\KAPE\\E\\$Extend\\$J","_offset":34464,"record_length":88,"major_version":2,"minor_version":0,"file_reference":{"entry":114704,"sequence":2},"parent_reference":{"entry":202493,"sequence":3},"usn":1231062688,"timestamp":"2018-07-30T20:15:57.100221Z","reason":"USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":32,"file_name_length":24,"file_name_offset":60,"file_name":"settings.dat"}
{"_source":"D:\\Images\\CTF_DEFCON_2018\\Image3-Desktop\\KAPE\\E\\$Extend\\$J","_offset":34368,"record_length":96,"major_version":2,"minor_version":0,"file_reference":{"entry":114893,"sequence":2},"parent_reference":{"entry":202493,"sequence":3},"usn":1231062592,"timestamp":"2018-07-30T20:15:57.100221Z","reason":"USN_REASON_DATA_OVERWRITE","source_info":"(empty)","security_id":0,"file_attributes":38,"file_name_length":34,"file_name_offset":60,"file_name":"settings.dat.LOG1"}
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

# Change Log
## [1.1.0] - 2019-06-01
### Added
 - `_source` to output
 - directory processing

## [1.0.0] - 2019-05-27
### Changed
- Rewrite and removal of features