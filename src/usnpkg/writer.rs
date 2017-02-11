use usnpkg::usn;

pub struct Writer {
    int_flags: bool
}

impl Writer {
    pub fn new(int_flags: bool) -> Writer {
        Writer {
            int_flags: int_flags
        }
    }

    pub fn write_header(&mut self) {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            "offset",
            "record_length",
            "major_version",
            "minor_version",
            "file_reference_number",
            "parent_file_reference_number",
            "usn",
            "timestamp",
            "reason",
            "source_info",
            "security_id",
            "file_attributes",
            "file_name_length",
            "file_name_offset",
            "file_name"
        );
    }

    pub fn write_record(&mut self, record: usn::UsnRecordV2, offset: u64) {
        if self.int_flags{
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                offset,
                record.record_length,
                record.major_version,
                record.minor_version,
                record.file_reference_number,
                record.parent_file_reference_number,
                record.usn,
                record.timestamp,
                record.reason,
                record.source_info,
                record.security_id,
                record.file_attributes,
                record.file_name_length,
                record.file_name_offset,
                record.file_name
            );
        } else {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}\t{}\t{}",
                offset,
                record.record_length,
                record.major_version,
                record.minor_version,
                record.file_reference_number,
                record.parent_file_reference_number,
                record.usn,
                record.timestamp,
                record.reason,
                record.source_info,
                record.security_id,
                record.file_attributes,
                record.file_name_length,
                record.file_name_offset,
                record.file_name
            );
        }
    }
}
