use thiserror::Error;

use super::{BlockVersion, ClientId};
use crate::blockstore::BlockId;

#[derive(Error, Debug)]
pub enum IntegrityViolationError {
    #[error(
        "Integrity Violation: Tried to roll back block {block:?} from client {from_client:?} version {from_version:?} to client {to_client:?} version {to_version:?}."
    )]
    RollBack {
        block: BlockId,
        from_client: ClientId,
        to_client: ClientId,
        from_version: BlockVersion,
        to_version: BlockVersion,
    },

    #[error("Integrity Violation: Block {id_from_header:?} is stored as block {id_from_filename:?}. Did an attacker try to rename some blocks?")]
    WrongBlockId {
        id_from_filename: BlockId,
        id_from_header: BlockId,
    },

    #[error("Integrity Violation: Block {block:?} should exist but we didn't find it. Did an attacker delete it?")]
    MissingBlock { block: BlockId },
}