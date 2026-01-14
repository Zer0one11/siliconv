#![allow(missing_docs)] // We don't need docs for each and every format, these aren't formats
// users will edit, just export to.

use std::io::{Read, Seek};

use siliconv_core::{
    error::ReplayError,
    replay::{Replay, ReplaySerializable},
};

pub mod silicate;

pub struct DynamicReplay(pub Replay);

impl DynamicReplay {
    /// Read a replay and try to automatically determine its format.
    ///
    /// The hint is the extension of the file.
    ///
    /// # Errors
    /// - If the format can't be determined
    pub fn read<R: Read + Seek>(reader: &mut R, hint: &str) -> Result<Self, ReplayError> {
        tracing::debug!("reading replay with hint {}", hint);

        match hint {
            "slc" => Ok(DynamicReplay(
                silicate::SilicateReplay::read(reader)?.into_replay(),
            )),
            _ => Err(ReplayError::ReadError(
                "could not determine format".to_string(),
            )),
        }
    }
}
