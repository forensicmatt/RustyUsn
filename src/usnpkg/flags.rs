use std::fmt;

bitflags! {
    pub flags Reason: u32 {
        const USN_REASON_BASIC_INFO_CHANGE      = 0x00008000,
        const USN_REASON_CLOSE                  = 0x80000000,
        const USN_REASON_COMPRESSION_CHANGE     = 0x00020000,
        const USN_REASON_DATA_EXTEND            = 0x00000002,
        const USN_REASON_DATA_OVERWRITE         = 0x00000001,
        const USN_REASON_DATA_TRUNCATION        = 0x00000004,
        const USN_REASON_EA_CHANGE              = 0x00000400,
        const USN_REASON_ENCRYPTION_CHANGE      = 0x00040000,
        const USN_REASON_FILE_CREATE            = 0x00000100,
        const USN_REASON_FILE_DELETE            = 0x00000200,
        const USN_REASON_HARD_LINK_CHANGE       = 0x00010000,
        const USN_REASON_INDEXABLE_CHANGE       = 0x00004000,
        const USN_REASON_INTEGRITY_CHANGE       = 0x00800000,
        const USN_REASON_NAMED_DATA_EXTEND      = 0x00000020,
        const USN_REASON_NAMED_DATA_OVERWRITE   = 0x00000010,
        const USN_REASON_NAMED_DATA_TRUNCATION  = 0x00000040,
        const USN_REASON_OBJECT_ID_CHANGE       = 0x00080000,
        const USN_REASON_RENAME_NEW_NAME        = 0x00002000,
        const USN_REASON_RENAME_OLD_NAME        = 0x00001000,
        const USN_REASON_REPARSE_POINT_CHANGE   = 0x00100000,
        const USN_REASON_SECURITY_CHANGE        = 0x00000800,
        const USN_REASON_STREAM_CHANGE          = 0x00200000,
        const USN_REASON_TRANSACTED_CHANGE      = 0x00400000,
    }
}
bitflags! {
    pub flags SourceInfo: u32 {
        const USN_SOURCE_AUXILIARY_DATA                 = 0x00000002,
        const USN_SOURCE_DATA_MANAGEMENT                = 0x00000001,
        const USN_SOURCE_REPLICATION_MANAGEMENT         = 0x00000004,
        const USN_SOURCE_CLIENT_REPLICATION_MANAGEMENT  = 0x00000008,
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.bits())
    }
}

impl fmt::Display for SourceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.bits())
    }
}
