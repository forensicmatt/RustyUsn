use std::fmt;
use serde::ser;


bitflags! {
    pub struct FileAttributes: u32 {
        const FILE_ATTRIBUTE_ARCHIVE                = 0x0000_0020;
        const FILE_ATTRIBUTE_COMPRESSED             = 0x0000_0800;
        const FILE_ATTRIBUTE_DEVICE                 = 0x0000_0040;
        const FILE_ATTRIBUTE_DIRECTORY              = 0x0000_0010;
        const FILE_ATTRIBUTE_ENCRYPTED              = 0x0000_4000;
        const FILE_ATTRIBUTE_HIDDEN                 = 0x0000_0002;
        const FILE_ATTRIBUTE_INTEGRITY_STREAM       = 0x0000_8000;
        const FILE_ATTRIBUTE_NORMAL                 = 0x0000_0080;
        const FILE_ATTRIBUTE_NOT_CONTENT_INDEXED    = 0x0000_2000;
        const FILE_ATTRIBUTE_NO_SCRUB_DATA          = 0x0002_0000;
        const FILE_ATTRIBUTE_OFFLINE                = 0x0000_1000;
        const FILE_ATTRIBUTE_READONLY               = 0x0000_0001;
        const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS  = 0x0040_0000;
        const FILE_ATTRIBUTE_RECALL_ON_OPEN         = 0x0004_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT          = 0x0000_0400;
        const FILE_ATTRIBUTE_SPARSE_FILE            = 0x0000_0200;
        const FILE_ATTRIBUTE_SYSTEM                 = 0x0000_0004;
        const FILE_ATTRIBUTE_TEMPORARY              = 0x0000_0100;
        const FILE_ATTRIBUTE_VIRTUAL                = 0x0000_1000;
    }
}
bitflags! {
    pub struct Reason: u32 {
        const USN_REASON_BASIC_INFO_CHANGE      = 0x0000_8000;
        const USN_REASON_CLOSE                  = 0x8000_0000;
        const USN_REASON_COMPRESSION_CHANGE     = 0x0002_0000;
        const USN_REASON_DATA_EXTEND            = 0x0000_0002;
        const USN_REASON_DATA_OVERWRITE         = 0x0000_0001;
        const USN_REASON_DATA_TRUNCATION        = 0x0000_0004;
        const USN_REASON_EA_CHANGE              = 0x0000_0400;
        const USN_REASON_ENCRYPTION_CHANGE      = 0x0004_0000;
        const USN_REASON_FILE_CREATE            = 0x0000_0100;
        const USN_REASON_FILE_DELETE            = 0x0000_0200;
        const USN_REASON_HARD_LINK_CHANGE       = 0x0001_0000;
        const USN_REASON_INDEXABLE_CHANGE       = 0x0000_4000;
        const USN_REASON_INTEGRITY_CHANGE       = 0x0080_0000;
        const USN_REASON_NAMED_DATA_EXTEND      = 0x0000_0020;
        const USN_REASON_NAMED_DATA_OVERWRITE   = 0x0000_0010;
        const USN_REASON_NAMED_DATA_TRUNCATION  = 0x0000_0040;
        const USN_REASON_OBJECT_ID_CHANGE       = 0x0008_0000;
        const USN_REASON_RENAME_NEW_NAME        = 0x0000_2000;
        const USN_REASON_RENAME_OLD_NAME        = 0x0000_1000;
        const USN_REASON_REPARSE_POINT_CHANGE   = 0x0010_0000;
        const USN_REASON_SECURITY_CHANGE        = 0x0000_0800;
        const USN_REASON_STREAM_CHANGE          = 0x0020_0000;
        const USN_REASON_TRANSACTED_CHANGE      = 0x0040_0000;
    }
}
bitflags! {
    pub struct SourceInfo: u32 {
        const USN_SOURCE_AUXILIARY_DATA                 = 0x0000_0002;
        const USN_SOURCE_DATA_MANAGEMENT                = 0x0000_0001;
        const USN_SOURCE_REPLICATION_MANAGEMENT         = 0x0000_0004;
        const USN_SOURCE_CLIENT_REPLICATION_MANAGEMENT  = 0x0000_0008;
    }
}

impl fmt::Display for FileAttributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.bits())
    }
}

impl ser::Serialize for FileAttributes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.bits())
    }
}

impl ser::Serialize for Reason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl fmt::Display for SourceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.bits())
    }
}

impl ser::Serialize for SourceInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}