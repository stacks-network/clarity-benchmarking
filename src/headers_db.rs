use std::{fs, io, path::PathBuf};

use stackslib::{
    chainstate::stacks::db::{MinerPaymentSchedule, StacksHeaderInfo, StacksBlockHeaderTypes},
    types::chainstate::{BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockId, VRFSeed},
    clarity::vm::database::HeadersDB,
};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use stackslib::chainstate::burn::ConsensusHash;
use stackslib::chainstate::stacks::index::ClarityMarfTrieId;
use stackslib::clarity_vm::database::get_matured_reward_pub;
use stackslib::util_lib::db::FromRow;
use stackslib::clarity::util::hash::Hash160;

pub struct TestHeadersDB;

impl HeadersDB for TestHeadersDB {
    fn get_stacks_block_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BlockHeaderHash> {
        Some(BlockHeaderHash(id_bhh.0.clone()))
    }

    fn get_burn_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BurnchainHeaderHash> {
        Some(BurnchainHeaderHash(id_bhh.0.clone()))
    }

    fn get_consensus_hash_for_block(&self, id_bhh: &StacksBlockId) -> Option<ConsensusHash> {
        let hash_bytes = Hash160::from_data(&id_bhh.0);
        Some(ConsensusHash(hash_bytes.0))
    }

    fn get_vrf_seed_for_block(&self, _id_bhh: &StacksBlockId) -> Option<VRFSeed> {
        Some(VRFSeed([0; 32]))
    }

    fn get_burn_block_time_for_block(&self, _id_bhh: &StacksBlockId) -> Option<u64> {
        Some(1)
    }

    fn get_burn_block_height_for_block(&self, id_bhh: &StacksBlockId) -> Option<u32> {
        if id_bhh == &StacksBlockId::sentinel() {
            Some(0)
        } else {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&id_bhh.0[0..4]);
            let height = u32::from_le_bytes(bytes);
            Some(height)
        }
    }

    fn get_miner_address(&self, _id_bhh: &StacksBlockId) -> Option<StacksAddress> {
        Some(StacksAddress::new(0, Hash160([0u8; 20])))
    }

    fn get_burnchain_tokens_spent_for_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        Some(0)
    }

    fn get_burnchain_tokens_spent_for_winning_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        Some(0)
    }

    fn get_tokens_earned_for_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        Some(0)
    }
}

pub struct SimHeadersDB {
    conn: Connection,
}

impl SimHeadersDB {
    pub fn new() -> Self {
        let db_path = "./new-db/index.sqlite";

        Self::new_with_path(db_path)
    }

    fn new_with_path(db_path: &str) -> Self {
        let open_flags = match fs::metadata(&db_path) {
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    // need to create
                    if let Some(dirp) = PathBuf::from(db_path).parent() {
                        fs::create_dir_all(dirp).unwrap_or_else(|e| {
                            panic!("Failed to create {:?}: {:?}", dirp, &e);
                        });
                    }
                    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
                } else {
                    panic!("FATAL: could not stat {}", db_path);
                }
            }
            Ok(_md) => OpenFlags::SQLITE_OPEN_READ_WRITE,
        };

        let conn = Connection::open_with_flags(db_path, open_flags).unwrap();

        SimHeadersDB { conn }
    }
}

impl HeadersDB for SimHeadersDB {
    fn get_stacks_block_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BlockHeaderHash> {
        get_stacks_header_info(&self.conn, id_bhh).map(|x| x.anchored_header.block_hash())
    }

    fn get_burn_header_hash_for_block(
        &self,
        id_bhh: &StacksBlockId,
    ) -> Option<BurnchainHeaderHash> {
        get_stacks_header_info(&self.conn, id_bhh).map(|x| x.burn_header_hash)
    }

    fn get_burn_block_time_for_block(&self, id_bhh: &StacksBlockId) -> Option<u64> {
        get_stacks_header_info(&self.conn, id_bhh).map(|x| x.burn_header_timestamp)
    }

    fn get_burn_block_height_for_block(&self, id_bhh: &StacksBlockId) -> Option<u32> {
        get_stacks_header_info(&self.conn, id_bhh).map(|x| x.burn_header_height)
    }

    fn get_vrf_seed_for_block(&self, id_bhh: &StacksBlockId) -> Option<VRFSeed> {
        get_stacks_header_info(&self.conn, id_bhh)
            .map(|x| match x.anchored_header {
                StacksBlockHeaderTypes::Epoch2(h) => VRFSeed::from_proof(&h.proof),
                StacksBlockHeaderTypes::Nakamoto(h) => todo!("Nakamoto blocks not supported yet"),
            })
    }

    fn get_miner_address(&self, id_bhh: &StacksBlockId) -> Option<StacksAddress> {
        get_miner_info(&self.conn, id_bhh).map(|x| x.address)
    }

    fn get_consensus_hash_for_block(&self, id_bhh: &StacksBlockId) -> Option<ConsensusHash> {
        get_stacks_header_info(&self.conn, id_bhh).map(|x| x.consensus_hash)
    }

    fn get_burnchain_tokens_spent_for_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        get_miner_info(&self.conn, id_bhh).map(|x| x.burnchain_sortition_burn.into())
    }

    fn get_burnchain_tokens_spent_for_winning_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        get_miner_info(&self.conn, id_bhh).map(|x| x.burnchain_commit_burn.into())
    }

    fn get_tokens_earned_for_block(&self, id_bhh: &StacksBlockId) -> Option<u128> {
        get_matured_reward_pub(&self.conn, id_bhh).map(|x| x.total().into())
    }
}

fn get_stacks_header_info(conn: &Connection, id_bhh: &StacksBlockId) -> Option<StacksHeaderInfo> {
    conn.query_row(
        "SELECT * FROM block_headers WHERE index_block_hash = ?",
        [id_bhh].iter(),
        |x| Ok(StacksHeaderInfo::from_row(x).expect("Bad stacks header info in database")),
    )
    .optional()
    .expect("Unexpected SQL failure querying block header table")
}

fn get_miner_info(conn: &Connection, id_bhh: &StacksBlockId) -> Option<MinerPaymentSchedule> {
    conn.query_row(
        "SELECT * FROM payments WHERE index_block_hash = ? AND miner = 1",
        [id_bhh].iter(),
        |x| Ok(MinerPaymentSchedule::from_row(x).expect("Bad payment info in database")),
    )
    .optional()
    .expect("Unexpected SQL failure querying payment table")
}
