use serde::Serialize;
use byteorder::{ByteOrder, LittleEndian};
use crate::liveusn::error::UsnLiveError;


/// This structure represents a NTFS_VOLUME_DATA_BUFFER structure
/// https://msdn.microsoft.com/en-us/windows/desktop/aa365256
/// 96 Bytes
#[derive(Serialize, Debug)]
pub struct NtfsVolumeData {
    pub volume_serial_number: i64,
    pub number_sectors: i64,
    pub total_clusters: i64,
    pub free_clusters: i64,
    pub total_reserved: i64,
    pub bytes_per_sector: u32,
    pub bytes_per_cluster: u32,
    pub bytes_per_file_record_segment: u32,
    pub clusters_per_file_record_segment: u32,
    pub mft_valid_data_length: i64,
    pub mft_start_lcn: i64,
    pub mft_2_start_lcn: i64,
    pub mft_zone_start: i64,
    pub mft_zone_end: i64,
    pub ntfs_extended_volume_data: Option<NtfsExtendedVolumeData>,
}
impl NtfsVolumeData {
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let buffer_size = buffer.len();
        let volume_serial_number = LittleEndian::read_i64(&buffer[0..8]);
        let number_sectors = LittleEndian::read_i64(&buffer[8..16]);
        let total_clusters = LittleEndian::read_i64(&buffer[16..24]);
        let free_clusters = LittleEndian::read_i64(&buffer[24..32]);
        let total_reserved = LittleEndian::read_i64(&buffer[32..40]);
        let bytes_per_sector = LittleEndian::read_u32(&buffer[40..44]);
        let bytes_per_cluster = LittleEndian::read_u32(&buffer[44..48]);
        let bytes_per_file_record_segment = LittleEndian::read_u32(&buffer[48..52]);
        let clusters_per_file_record_segment = LittleEndian::read_u32(&buffer[52..56]);
        let mft_valid_data_length = LittleEndian::read_i64(&buffer[56..64]);
        let mft_start_lcn = LittleEndian::read_i64(&buffer[64..72]);
        let mft_2_start_lcn = LittleEndian::read_i64(&buffer[72..80]);
        let mft_zone_start = LittleEndian::read_i64(&buffer[80..88]);
        let mft_zone_end = LittleEndian::read_i64(&buffer[88..96]);

        let mut ntfs_extended_volume_data = None;
        if buffer_size >= 128 as usize {
            ntfs_extended_volume_data = Some(
                NtfsExtendedVolumeData::from_buffer(
                    &buffer[96..]
                )
            )
        }

        NtfsVolumeData {
            volume_serial_number,
            number_sectors,
            total_clusters,
            free_clusters,
            total_reserved,
            bytes_per_sector,
            bytes_per_cluster,
            bytes_per_file_record_segment,
            clusters_per_file_record_segment,
            mft_valid_data_length,
            mft_start_lcn,
            mft_2_start_lcn,
            mft_zone_start,
            mft_zone_end,
            ntfs_extended_volume_data
        }
    }
}


/// This structure represents a NTFS_EXTENDED_VOLUME_DATA structure
/// https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-ntfs_extended_volume_data
/// 32 Bytes
#[derive(Serialize, Debug)]
pub struct NtfsExtendedVolumeData {
    pub byte_count: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub bytes_per_physical_sector: u32,
    pub lfs_major_version: u16,
    pub lfs_minor_version: u16,
    pub max_device_trim_extent_count: u32,
    pub max_device_trim_byte_count: u32,
    pub max_volume_trim_extent_count: u32,
    pub max_volume_trim_byte_count: u32,
}
impl NtfsExtendedVolumeData {
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let byte_count = LittleEndian::read_u32(&buffer[0..4]);
        let major_version = LittleEndian::read_u16(&buffer[4..6]);
        let minor_version = LittleEndian::read_u16(&buffer[6..8]);
        let bytes_per_physical_sector = LittleEndian::read_u32(&buffer[8..12]);
        let lfs_major_version = LittleEndian::read_u16(&buffer[12..14]);
        let lfs_minor_version = LittleEndian::read_u16(&buffer[14..16]);
        let max_device_trim_extent_count = LittleEndian::read_u32(&buffer[16..20]);
        let max_device_trim_byte_count = LittleEndian::read_u32(&buffer[20..24]);
        let max_volume_trim_extent_count = LittleEndian::read_u32(&buffer[24..28]);
        let max_volume_trim_byte_count = LittleEndian::read_u32(&buffer[28..32]);

        NtfsExtendedVolumeData {
            byte_count,
            major_version,
            minor_version,
            bytes_per_physical_sector,
            lfs_major_version,
            lfs_minor_version,
            max_device_trim_extent_count,
            max_device_trim_byte_count,
            max_volume_trim_extent_count,
            max_volume_trim_byte_count,
        }
    }
}


/// Wrapper for the different USN_JOURNAL_DATA versions.
#[derive(Debug, Clone)]
pub enum UsnJournalData {
    V0(UsnJournalDataV0),
    V1(UsnJournalDataV1),
    V2(UsnJournalDataV2)
}
impl UsnJournalData {
    pub fn new(buffer: &[u8]) -> Result<UsnJournalData, UsnLiveError> {
        match buffer.len() {
            56 => {
                return Ok(
                    UsnJournalData::V0(
                        UsnJournalDataV0::new(&buffer)
                    )
                );
            },
            60 => {
                return Ok(
                    UsnJournalData::V1(
                        UsnJournalDataV1::new(&buffer)
                    )
                );
            },
            80 => {
                return Ok(
                    UsnJournalData::V2(
                        UsnJournalDataV2::new(&buffer)
                    )
                );
            },
            other => {
                return Err(
                    UsnLiveError::invalid_usn_journal_data(other)
                );
            }
        }
    }

    pub fn get_next_usn(&self) -> u64 {
        match self {
            UsnJournalData::V0(jd) => jd.next_usn,
            UsnJournalData::V1(jd) => jd.next_usn,
            UsnJournalData::V2(jd) => jd.next_usn,
        }
    }
}


/// Represents a USN_JOURNAL_DATA_V0 structure
/// https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_journal_data_v0
/// Size 56
#[derive(Debug, Clone)]
pub struct UsnJournalDataV0 {
    usn_jounral_id: u64,
    first_usn: u64,
    next_usn: u64,
    lowest_valid_usn: u64,
    max_usn: u64,
    maximum_size: u64,
    allocation_delta: u64,
}
impl UsnJournalDataV0 {
    fn new(buffer: &[u8]) -> UsnJournalDataV0 {
        let usn_jounral_id = LittleEndian::read_u64(&buffer[0..8]);
        let first_usn = LittleEndian::read_u64(&buffer[8..16]);
        let next_usn = LittleEndian::read_u64(&buffer[16..24]);
        let lowest_valid_usn = LittleEndian::read_u64(&buffer[24..32]);
        let max_usn = LittleEndian::read_u64(&buffer[32..40]);
        let maximum_size = LittleEndian::read_u64(&buffer[40..48]);
        let allocation_delta = LittleEndian::read_u64(&buffer[48..56]);

        return UsnJournalDataV0 {
            usn_jounral_id,
            first_usn,
            next_usn,
            lowest_valid_usn,
            max_usn,
            maximum_size,
            allocation_delta,
        }
    }
}


/// Represents a USN_JOURNAL_DATA_V1 structure
/// https://docs.microsoft.com/en-us/windows/desktop/api/winioctl/ns-winioctl-usn_journal_data_v1
/// Size 60
#[derive(Debug, Clone)]
pub struct UsnJournalDataV1 {
    usn_jounral_id: u64,
    first_usn: u64,
    next_usn: u64,
    lowest_valid_usn: u64,
    max_usn: u64,
    maximum_size: u64,
    allocation_delta: u64,
    min_major_version: u16,
    max_major_version: u16,
}
impl UsnJournalDataV1 {
    fn new(buffer: &[u8]) -> UsnJournalDataV1 {
        let usn_jounral_id = LittleEndian::read_u64(&buffer[0..8]);
        let first_usn = LittleEndian::read_u64(&buffer[8..16]);
        let next_usn = LittleEndian::read_u64(&buffer[16..24]);
        let lowest_valid_usn = LittleEndian::read_u64(&buffer[24..32]);
        let max_usn = LittleEndian::read_u64(&buffer[32..40]);
        let maximum_size = LittleEndian::read_u64(&buffer[40..48]);
        let allocation_delta = LittleEndian::read_u64(&buffer[48..56]);
        let min_major_version = LittleEndian::read_u16(&buffer[56..58]);
        let max_major_version = LittleEndian::read_u16(&buffer[58..60]);

        return UsnJournalDataV1 {
            usn_jounral_id,
            first_usn,
            next_usn,
            lowest_valid_usn,
            max_usn,
            maximum_size,
            allocation_delta,
            min_major_version,
            max_major_version,
        }
    }
}


/// Represents a USN_JOURNAL_DATA_V2 structure
/// https://docs.microsoft.com/en-us/windows/desktop/api/winioctl/ns-winioctl-usn_journal_data_v2
/// Size 80
#[derive(Debug, Clone)]
pub struct UsnJournalDataV2 {
    usn_jounral_id: u64,
    first_usn: u64,
    next_usn: u64,
    lowest_valid_usn: u64,
    max_usn: u64,
    maximum_size: u64,
    allocation_delta: u64,
    min_major_version: u16,
    max_major_version: u16,
    flags: u32,
    range_track_chunk_size: u64,
    range_track_file_size_threshold: i64,
}
impl UsnJournalDataV2 {
    fn new(buffer: &[u8]) -> UsnJournalDataV2 {
        let usn_jounral_id = LittleEndian::read_u64(&buffer[0..8]);
        let first_usn = LittleEndian::read_u64(&buffer[8..16]);
        let next_usn = LittleEndian::read_u64(&buffer[16..24]);
        let lowest_valid_usn = LittleEndian::read_u64(&buffer[24..32]);
        let max_usn = LittleEndian::read_u64(&buffer[32..40]);
        let maximum_size = LittleEndian::read_u64(&buffer[40..48]);
        let allocation_delta = LittleEndian::read_u64(&buffer[48..56]);
        let min_major_version = LittleEndian::read_u16(&buffer[56..58]);
        let max_major_version = LittleEndian::read_u16(&buffer[58..60]);
        let flags = LittleEndian::read_u32(&buffer[60..64]);
        let range_track_chunk_size = LittleEndian::read_u64(&buffer[64..72]);
        let range_track_file_size_threshold = LittleEndian::read_i64(&buffer[72..80]);

        return UsnJournalDataV2 {
            usn_jounral_id,
            first_usn,
            next_usn,
            lowest_valid_usn,
            max_usn,
            maximum_size,
            allocation_delta,
            min_major_version,
            max_major_version,
            flags,
            range_track_chunk_size,
            range_track_file_size_threshold,
        }
    }
}


/// Wrapper for the different READ_USN_JOURNAL_DATA versions.
#[derive(Debug, Clone)]
pub enum ReadUsnJournalData {
    V0(ReadUsnJournalDataV0),
    V1(ReadUsnJournalDataV1),
}
impl ReadUsnJournalData {
    pub fn from_usn_journal_data(journal_data: UsnJournalData) -> ReadUsnJournalData {
        match journal_data {
            UsnJournalData::V0(journal_data_v0) => {
                return ReadUsnJournalData::V0(
                    ReadUsnJournalDataV0::new(
                        journal_data_v0.first_usn,
                        journal_data_v0.usn_jounral_id
                    )
                );
            },
            UsnJournalData::V1(journal_data_v1) => {
                return ReadUsnJournalData::V1(
                    ReadUsnJournalDataV1::new(
                        journal_data_v1.first_usn,
                        journal_data_v1.usn_jounral_id,
                        journal_data_v1.min_major_version,
                        journal_data_v1.max_major_version
                    )
                );
            },
            UsnJournalData::V2(journal_data_v2) => {
                return ReadUsnJournalData::V1(
                    ReadUsnJournalDataV1::new(
                        journal_data_v2.first_usn,
                        journal_data_v2.usn_jounral_id,
                        journal_data_v2.min_major_version,
                        journal_data_v2.max_major_version
                    )
                );
            }
        }
    }

    pub fn with_reason_mask(mut self, reason_mask: u32) -> Self {
        match self {
            ReadUsnJournalData::V0(ref mut read_data_v0) => {
                read_data_v0.reason_mask = reason_mask
            },
            ReadUsnJournalData::V1(ref mut read_data_v1) => {
                read_data_v1.reason_mask = reason_mask
            }
        }

        self
    }

    pub fn with_start_usn(mut self, start_usn: u64) -> Self {
        match self {
            ReadUsnJournalData::V0(ref mut read_data_v0) => {
                read_data_v0.start_usn = start_usn
            },
            ReadUsnJournalData::V1(ref mut read_data_v1) => {
                read_data_v1.start_usn = start_usn
            }
        }

        self
    }
}


/// Represents a READ_USN_JOURNAL_DATA_V0 structure
/// https://docs.microsoft.com/en-us/windows/desktop/api/winioctl/ns-winioctl-read_usn_journal_data_v0
/// Size 40
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ReadUsnJournalDataV0 {
    start_usn: u64,
    reason_mask: u32,
    return_only_on_close: u32,
    timeout: u64,
    bytes_to_wait_for: u64,
    usn_journal_id: u64,
}
impl ReadUsnJournalDataV0 {
    fn new(start_usn: u64, usn_journal_id: u64) -> ReadUsnJournalDataV0 {
        let reason_mask = 0xffffffff;
        let return_only_on_close = 0;
        let timeout = 0;
        let bytes_to_wait_for = 0;

        return ReadUsnJournalDataV0 {
            start_usn,
            reason_mask,
            return_only_on_close,
            timeout,
            bytes_to_wait_for,
            usn_journal_id,
        }
    }
}


/// Represents a READ_USN_JOURNAL_DATA_V1 structure
/// https://docs.microsoft.com/en-us/windows/desktop/api/winioctl/ns-winioctl-read_usn_journal_data_v1
/// Size 44
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ReadUsnJournalDataV1 {
    start_usn: u64,
    reason_mask: u32,
    return_only_on_close: u32,
    timeout: u64,
    bytes_to_wait_for: u64,
    usn_journal_id: u64,
    min_major_version: u16,
    max_major_version: u16,
}
impl ReadUsnJournalDataV1 {
    fn new(
        start_usn: u64, usn_journal_id: u64, 
        min_major_version: u16, max_major_version: u16
    ) -> ReadUsnJournalDataV1 {
        let reason_mask = 0xffffffff;
        let return_only_on_close = 0;
        let timeout = 0;
        let bytes_to_wait_for = 0;

        return ReadUsnJournalDataV1 {
            start_usn,
            reason_mask,
            return_only_on_close,
            timeout,
            bytes_to_wait_for,
            usn_journal_id,
            min_major_version,
            max_major_version,
        }
    }
}
