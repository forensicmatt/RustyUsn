use std::ptr;
use std::mem;
use std::fs::File;
use winapi::um::winioctl::{
    FSCTL_QUERY_USN_JOURNAL,
    FSCTL_READ_USN_JOURNAL,
    FSCTL_GET_NTFS_FILE_RECORD,
    FSCTL_GET_NTFS_VOLUME_DATA,
    NTFS_FILE_RECORD_INPUT_BUFFER
};
use winapi::ctypes::c_void;
use winapi::um::winnt::LARGE_INTEGER;
use std::os::windows::io::AsRawHandle;
use winapi::um::ioapiset::DeviceIoControl;
use crate::liveusn::error::UsnLiveError;
use crate::liveusn::ntfs;


/// Query FSCTL_GET_NTFS_VOLUME_DATA to get the NTFS volume data.
/// https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_get_ntfs_volume_data
/// 
pub fn get_ntfs_volume_data(volume_handle: &File) -> Result<ntfs::NtfsVolumeData, UsnLiveError> {
    let mut bytes_read = 0;
    let mut output_buffer = vec![0u8; 128];

    let result = unsafe {
        DeviceIoControl(
            volume_handle.as_raw_handle() as *mut c_void,
            FSCTL_GET_NTFS_VOLUME_DATA,
            ptr::null_mut(),
            0,
            output_buffer.as_mut_ptr() as *mut _,
            output_buffer.len() as u32,
            &mut bytes_read,
            ptr::null_mut(),
        )
    };

    if result == 0 {
        return Err(
            UsnLiveError::from_windows_last_error()
        );
    }

    debug!("[output_buffer] DeviceIoControl->FSCTL_GET_NTFS_VOLUME_DATA: {}", hex::encode(&output_buffer));

    Ok(
        ntfs::NtfsVolumeData::from_buffer(
            &output_buffer[..]
        )
    )
}


/// Query FSCTL_GET_NTFS_FILE_RECORD to get an entries' NTFS_FILE_RECORD_OUTPUT_BUFFER
/// https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_get_ntfs_file_record
///
pub fn query_file_record(volume_handle: &File, entry: i64, entry_size: u32) -> Result<Vec<u8>, UsnLiveError> {
    let mut bytes_read = 0;
    let buffer_size = (entry_size + 12) as usize;
    let mut output_buffer = vec![0u8; buffer_size];

    let result = unsafe {
        let mut entry_reference = mem::zeroed::<LARGE_INTEGER>();
        *entry_reference.QuadPart_mut() = entry;

        // Input buffer
        let mut input_buffer = NTFS_FILE_RECORD_INPUT_BUFFER { 
            FileReferenceNumber: entry_reference 
        };

        DeviceIoControl(
            volume_handle.as_raw_handle() as *mut c_void,
            FSCTL_GET_NTFS_FILE_RECORD,
            &mut input_buffer as *mut _ as *mut c_void,
            mem::size_of::<NTFS_FILE_RECORD_INPUT_BUFFER>() as u32,
            output_buffer.as_mut_ptr() as *mut _,
            output_buffer.len() as u32,
            &mut bytes_read,
            ptr::null_mut()
        )
    };

    if result == 0 {
        return Err(
            UsnLiveError::from_windows_last_error()
        );
    } else {
        output_buffer.truncate(
            bytes_read as usize
        );
    }

    Ok(output_buffer)
}


/// Query FSCTL_QUERY_USN_JOURNAL to get UsnJournalData which is an enum for
/// READ_USN_JOURNAL_DATA_V0, READ_USN_JOURNAL_DATA_V1, READ_USN_JOURNAL_DATA_V2 structures.
/// https://docs.microsoft.com/en-us/windows/desktop/api/winioctl/ni-winioctl-fsctl_query_usn_journal
///
pub fn query_usn_journal(volume_handle: &File) -> Result<ntfs::UsnJournalData, UsnLiveError> {
    let mut output_buffer = [0u8; 80];
    let mut bytes_read = 0;

    let result = unsafe {
        DeviceIoControl(
            volume_handle.as_raw_handle(),
            FSCTL_QUERY_USN_JOURNAL,
            ptr::null_mut(),
            0,
            output_buffer.as_mut_ptr() as *mut _,
            output_buffer.len() as u32,
            &mut bytes_read,
            ptr::null_mut()
        )
    };

    if result == 0 {
        return Err(
            UsnLiveError::from_windows_last_error()
        );
    } else {
        return ntfs::UsnJournalData::new(
            &output_buffer[..bytes_read as usize]
        );
    }
}


/// Query FSCTL_READ_USN_JOURNAL
/// https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_read_usn_journal
/// 
pub fn read_usn_journal<'a> (
    volume_handle: &File, 
    read_jounral_data: ntfs::ReadUsnJournalData, 
    record_buffer: &'a mut [u8]
) -> Result<&'a [u8], UsnLiveError> {
    let mut bytes_read: u32 = 0;

    let result = match read_jounral_data {
        ntfs::ReadUsnJournalData::V0(mut read_data_v0) => {
            unsafe {
                DeviceIoControl(
                    volume_handle.as_raw_handle(),
                    FSCTL_READ_USN_JOURNAL,
                    &mut read_data_v0 as *mut _ as *mut c_void,
                    mem::size_of::<ntfs::ReadUsnJournalDataV0>() as u32,
                    record_buffer.as_mut_ptr() as *mut _,
                    record_buffer.len() as u32,
                    &mut bytes_read,
                    ptr::null_mut()
                )
            }
        },
        ntfs::ReadUsnJournalData::V1(mut read_data_v1) => {
            unsafe {
                DeviceIoControl(
                    volume_handle.as_raw_handle(),
                    FSCTL_READ_USN_JOURNAL,
                    &mut read_data_v1 as *mut _ as *mut c_void,
                    mem::size_of::<ntfs::ReadUsnJournalDataV1>() as u32,
                    record_buffer.as_mut_ptr() as *mut _,
                    record_buffer.len() as u32,
                    &mut bytes_read,
                    ptr::null_mut()
                )
            }
        },
    };

    if result == 0 {
        return Err(
            UsnLiveError::from_windows_last_error()
        );
    } else {
        return Ok(
            &record_buffer[..bytes_read as usize]
        )
    }
}
