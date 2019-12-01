use std::io;
use std::fmt;
use mft::MftParser;
use crate::ReadSeek;
use serde::Serialize;
use lru::LruCache;
use std::collections::HashMap;
use winstructs::ntfs::mft_reference::MftReference;
use serde::ser::{Serializer, SerializeMap};


#[derive(Serialize, Debug)]
pub struct EntryMapping {
    pub name: String,
    pub parent: MftReference,
}


pub struct FolderMapping {
    pub mapping: HashMap<MftReference, EntryMapping>,
    pub cache: LruCache<MftReference, String>
}

impl fmt::Debug for FolderMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FolderMapping {{ mapping: {:?}, cache: LruCache }}", self.mapping)
    }
}

impl FolderMapping {
    pub fn new() -> Self {
        let mapping: HashMap<MftReference, EntryMapping> = HashMap::new();
        let cache: LruCache<MftReference, String> = LruCache::new(100);

        FolderMapping {
            mapping,
            cache
        }
    }

    pub fn contains_reference(&self, entry_reference: &MftReference) -> bool {
        self.mapping.contains_key(
            entry_reference
        )
    }

    pub fn from_mft_path(filename: &str) -> Result<Self, io::Error> {
        let mapping: HashMap<MftReference, EntryMapping> = HashMap::new();
        let mut parser = MftParser::from_path(filename).unwrap();
        let cache: LruCache<MftReference, String> = LruCache::new(100);
        let mut folder_mapping = FolderMapping {
            mapping,
            cache
        };

        folder_mapping.build_folder_mapping(
            &mut parser
        );

        Ok(folder_mapping)
    }

    pub fn build_folder_mapping<T: ReadSeek>(&mut self, mft_parser: &mut MftParser<T>) {
        for entry in mft_parser.iter_entries() {
            match entry {
                Ok(e) =>  {
                    if e.is_dir() {
                        let mut l_entry = e.header.record_number;
                        let mut l_sequence = e.header.sequence;

                        if !e.is_allocated() {
                            l_sequence = l_sequence - 1;
                        }

                        // if entry is child, set entry and sequence to parent
                        if e.header.base_reference.entry != 0 {
                            l_entry = e.header.base_reference.entry;
                            l_sequence = e.header.base_reference.sequence;
                        }

                        let file_name_attr = match e.find_best_name_attribute() {
                            Some(fn_attr) => fn_attr,
                            None => continue
                        };

                        let entry_map = EntryMapping{
                            name: file_name_attr.name,
                            parent: file_name_attr.parent
                        };

                        let entry_reference = MftReference::new(
                            l_entry,
                            l_sequence
                        );

                        self.mapping.insert(
                            entry_reference,
                            entry_map
                        );
                    } 
                    else {
                        continue
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
    }

    pub fn remove_mapping(&mut self, entry_reference: MftReference) {
        self.mapping.remove(
            &entry_reference
        );
    }

    pub fn add_mapping(&mut self, entry_reference: MftReference, name: String, parent: MftReference) {
        let entry_map = EntryMapping {
            name: name,
            parent: parent
        };

        // If there is a cached entry for this reference, we need to remove it
        // so that it can be recreated with the new mapping.
        self.cache.pop(
            &entry_reference
        );

        self.mapping.insert(
            entry_reference,
            entry_map
        );
    }

    fn enumerate_path_queue(&self, lookup_ref: &MftReference, path_queue: &mut Vec<String>) {
        if lookup_ref.entry != 5 {
            match self.mapping.get(&lookup_ref) {
                Some(folder_map) => {
                    path_queue.push(folder_map.name.clone());

                    self.enumerate_path_queue(
                        &folder_map.parent,
                        path_queue
                    );
                },
                None => {
                    path_queue.push("[<unknown>]".to_string());
                }
            }
        } else {
            path_queue.push("[root]".to_string());
        }
    }

    pub fn enumerate_path(&mut self, entry: u64, sequence: u16) -> Option<String> {
        let lookup_ref = MftReference {
            entry, sequence
        };
        
        match self.cache.get_mut(&lookup_ref) {
            Some(full_path) => {
                return Some(full_path.clone());
            },
            None => {
                let mut path_queue: Vec<String> = Vec::new();

                self.enumerate_path_queue(
                    &lookup_ref, 
                    &mut path_queue
                );

                path_queue.reverse();
                let full_path = path_queue.join("/");

                self.cache.put(
                    lookup_ref, 
                    full_path.clone()
                );

                return Some(full_path);
            }
        }
    }
}

impl Serialize for FolderMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.mapping.len()))?;
        for (k, v) in &self.mapping {
            map.serialize_entry(
                &k.entry, &v
            )?;
        }
        map.end()
    }
}