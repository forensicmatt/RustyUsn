[![Build Status](https://dev.azure.com/matthewseyer/dfir/_apis/build/status/forensicmatt.RustyUsn?branchName=master)](https://dev.azure.com/matthewseyer/dfir/_build/latest?definitionId=1&branchName=master)
# RustyUsn
A fast and cross platform USN Parser written in Rust. Output is [JSONL](http://jsonlines.org/).

# Tools
There are currently two tools associated with this package. rusty_usn and listen_usn. Neither currently implement records for usn record version 3 and 4.

## rust_usn

```
rusty_usn 1.2.0
Matthew Seyer <https://github.com/forensicmatt/RustyUsn>
USN Parser written in Rust. Output is JSONL.

USAGE:
    rusty_usn.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --debug <DEBUG>        Debug level to use. [possible values: Off, Error, Warn, Info, Debug, Trace]
    -m, --mft <MFT>            The MFT to use for creating folder mapping.
    -s, --source <PATH>        The source to parse. If the source is a directory, the directoy will be recursed looking
                               for any files that end with '$J'. (Do not use a directory if using an MFT file.)
    -t, --threads <threads>    Sets the number of worker threads, defaults to number of CPU cores. If the --mft option
                               is used, the tool can only run single threaded. [default: 0]
```

### Output
Records are written to stdout as jsonl.

```
{"_offset":40018936,"_source":"C:\\Test\\$UsnJrnl.J","file_attributes":"FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM","file_name":"lastalive0.dat","file_name_length":28,"file_name_offset":60,"file_reference":{"entry":61346,"sequence":10},"full_name":"[root]/Windows/ServiceProfiles/LocalService/AppData/Local/lastalive0.dat","major_version":2,"minor_version":0,"parent_reference":{"entry":83529,"sequence":2},"reason":"USN_REASON_CLOSE | USN_REASON_DATA_EXTEND | USN_REASON_DATA_TRUNCATION","record_length":88,"security_id":0,"source_info":"(empty)","timestamp":"2019-03-20T21:35:52.322741Z","usn":558015480}
{"_offset":40018848,"_source":"C:\\Test\\$UsnJrnl.J","file_attributes":"FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM","file_name":"lastalive0.dat","file_name_length":28,"file_name_offset":60,"file_reference":{"entry":61346,"sequence":10},"full_name":"[root]/Windows/ServiceProfiles/LocalService/AppData/Local/lastalive0.dat","major_version":2,"minor_version":0,"parent_reference":{"entry":83529,"sequence":2},"reason":"USN_REASON_DATA_EXTEND | USN_REASON_DATA_TRUNCATION","record_length":88,"security_id":0,"source_info":"(empty)","timestamp":"2019-03-20T21:35:52.322741Z","usn":558015392}
```

## listen_usn
A tool that uses the Windows API to listen to USN changes for a given volume in real-time. Output is JSONL. Note 
that this tools requires the "windows" feature (which is not on by default) to be built. This is required for the build 
process to complete on non-windows platforms. (see the **build** section of this README)

Also note, the _offset field in output is currently the value of the buffer returned by the Windows API. Don't be supprised to see lots of the same offset for this tool's output.

```
listen_usn 0.1.0
Matthew Seyer <https://github.com/forensicmatt/RustyUsn>
USN listener written in Rust. Output is JSONL.

USAGE:
    listen_usn.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -p, --historical    List historical records along with listening to new changes.
    -V, --version       Prints version information

OPTIONS:
    -d, --debug <DEBUG>    Debug level to use. [possible values: Off, Error, Warn, Info, Debug, Trace]
    -s, --source <PATH>    The source volume to listen to. (example: '\\.\C:')
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
If you are building on windows and want `listen_usn.exe` you will need to build with the `windows` feature as it is not on by default. Use: `cargo build --all-features --release` for compiling with Rust in Windows. Use `cargo build --release` for non-Windows systems.

Currently using Rust 1.36.0 Nightly.
