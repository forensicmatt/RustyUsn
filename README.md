# RustyUsn
A fast USN Parser writen in Rust.

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
Here are some benchmarks ran on a USN Journal file that contains 367260 records and is 35.9 MB (37,687,192 bytes).

```
PS E:\RustyUsn\target\release> Measure-Command {.\RustyUsn.exe -j E:\Testing\UsnJrnl.J}

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 6
Milliseconds      : 865
Ticks             : 68655650
TotalDays         : 7.94625578703704E-05
TotalHours        : 0.00190710138888889
TotalMinutes      : 0.114426083333333
TotalSeconds      : 6.865565
TotalMilliseconds : 6865.565
```

```
PS E:\RustyUsn\target\release> Measure-Command {type E:\Testing\UsnJrnl.J | .\RustyUsn.exe -p}

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 2
Milliseconds      : 883
Ticks             : 28837932
TotalDays         : 3.33772361111111E-05
TotalHours        : 0.000801053666666667
TotalMinutes      : 0.04806322
TotalSeconds      : 2.8837932
TotalMilliseconds : 2883.7932
```

Both output produce the same results for this file. It is obvious that there are some improvements to be made on the file io option. Currently there is no buffering in place which could improve the time of the fileio option faster.

## Carving
The idea is to beable to parse records from stdin. You can grab unallocated with the Sleuthkit's blkls. Currently this has failed with RustyUsn.exe dying in some tests. I think more error checks are needed.
```
blkls.exe -o OFFSET IMAGEPATH | RustyUsn.exe -p > carved_records.txt
```
