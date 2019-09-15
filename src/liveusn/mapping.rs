use lru::LruCache;
use crate::liveusn::error::UsnLiveError;
use crate::liveusn::live::WindowsLiveNtfs;


fn enumerate_path_stack(ntfs_live: &mut WindowsLiveNtfs, entry: i64, path_stack: &mut Vec<String>) -> Result<(), UsnLiveError> {
    let mft_entry = ntfs_live.get_entry(entry)?;

    if entry != mft_entry.header.record_number as i64 {
        panic!("Requested entry {}, but got back entry {}", 
            entry, mft_entry.header.record_number);
    }

    match mft_entry.find_best_name_attribute() {
        Some(attr) => {
            if attr.parent.entry != 5 {
                enumerate_path_stack(
                    ntfs_live,
                    attr.parent.entry as i64,
                    path_stack
                )?;
            }
            path_stack.push(attr.name);
        },
        None => {
            return Err(
                UsnLiveError::unable_to_get_name_attr(
                    &format!("Unable to retrieve filename attribute for entry: {}", entry)
                )
            );
        }
    }

    Ok(())
}

pub struct LiveMapping {
    pub ntfs_live: WindowsLiveNtfs,
    pub cache: LruCache<i64, String>
}

impl LiveMapping {
    pub fn from_volume_path(volume_path: &str) -> Result<Self, UsnLiveError> {
        let ntfs_live = WindowsLiveNtfs::from_volume_path(volume_path)?;
        let cache: LruCache<i64, String> = LruCache::new(1000);

        Ok(
            LiveMapping {
                ntfs_live,
                cache
            }
        )
    }

    pub fn remove_path_from_cache(&mut self, entry: i64){
        self.cache.pop(&entry);
    }

    /// Get the full path of an entry number.
    pub fn get_full_path(&mut self, entry: i64) -> Result<String, UsnLiveError> {
        match self.cache.get_mut(&entry) {
            Some(full_path) => {
                Ok(full_path.clone())
            },
            None => {
                let mut path_stack: Vec<String> = Vec::new();

                enumerate_path_stack(
                    &mut self.ntfs_live,
                    entry, 
                    &mut path_stack
                )?;

                let full_path = path_stack.join("/");

                self.cache.put(
                    entry, 
                    full_path.clone()
                );

                Ok(full_path)
            }
        }
    }

    /// Get the entry name for a given entry number.
    pub fn get_entry_name(&mut self, entry: i64) -> Result<String, UsnLiveError> {
        let mft_entry = self.ntfs_live.get_entry(entry)?;

        let fn_attr = match mft_entry.find_best_name_attribute() {
            Some(attr) => attr,
            None => {
                return Err(
                    UsnLiveError::unable_to_get_name_attr(
                        &format!("Unable to retrieve filename attribute for entry: {}", entry)
                    )
                );
            }
        };
        
        Ok(
            fn_attr.name
        )
    }
}
