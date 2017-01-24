# RustyUsn
A Rust USN Parser

Just trying to learn some rust...

Goals are to:
- [x] Learn how to parse command line arguments
- [x] Handle errors
- [x] Read into structs
- [ ] Datetime Timezone handling
- [x] Parse UFT-16
- [ ] Bind with python

## Example
```
target\debug\rusty_usn.exe -j testdata\record.usn
Journal to parse: testdata\record.usn
function: get_next_record() at offset: 0
USN structure 1: UsnRecordV2 {
    record_length: 96,
    major_version: 2,
    minor_version: 0,
    file_reference_number: 10477624533077459059,
    parent_file_reference_number: 1970324837116475,
    usn: 20342374400,
    timestamp: 2013-10-19T12:16:53.276040,
    reason: 2,
    source_info: 0,
    security_id: 0,
    file_attributes: 8224,
    file_name_length: 32,
    file_name_offset: 60,
    file_name: "BTDevManager.log"
}
```