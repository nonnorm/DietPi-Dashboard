use std::time::Duration;

use ephemeropt::EphemeralOption;
use proto::{backend::ResponseBackendMessage, frontend::RequestFrontendMessage};

const CACHE_DURATION: Duration = Duration::from_millis(1500);

macro_rules! cache {
    ($name:ident, [$($key:ident: $discrim:ident),*]) => {
        pub struct $name {
            $( $key: EphemeralOption<ResponseBackendMessage> ),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $( $key: EphemeralOption::new_empty(CACHE_DURATION), )*
                }
            }

            pub fn get(&self, key: &RequestFrontendMessage) -> Option<ResponseBackendMessage> {
                match key {
                    $( RequestFrontendMessage::$discrim => self.$key.get().cloned(), )*
                    _ => None,
                }
            }

            pub fn insert(&mut self, val: ResponseBackendMessage) {
                match val {
                    $( ResponseBackendMessage::$discrim(_) => { self.$key.insert(val); }, )*
                    _ => {}
                };
            }
        }
    };
}

cache!(BackendCache, [cpu: Cpu, temp: Temp, mem: Mem, disk: Disk, net_io: NetIO, processes: Processes]);
