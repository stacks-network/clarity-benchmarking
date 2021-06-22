use std::{fs, io, path::PathBuf};

use blockstack_lib::{
    chainstate::stacks::db::{MinerPaymentSchedule, StacksHeaderInfo},
    types::{
        chainstate::{BlockHeaderHash, BurnchainHeaderHash, StacksAddress, StacksBlockId, VRFSeed},
        proof::ClarityMarfTrieId,
    },
    util::{db::FromRow, hash::Hash160},
    vm::database::HeadersDB,
};

use rusqlite::{Connection, Error as sqlite_error, OpenFlags, OptionalExtension};

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
}

pub struct SimHeadersDB {
    conn: Connection,
}

impl SimHeadersDB {
    pub fn new() -> Self {
        let db_path = "../chainstate.sqlite";
        let metadata = fs::metadata(&db_path);

        let open_flags = match &metadata {
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

        let mut db = SimHeadersDB { conn };

        if metadata.is_err() {
            if db.instantiate().is_err() {
                panic!("FATAL: could not instantiate db");
            };
        }
        db
    }

    fn instantiate(&mut self) -> Result<(), sqlite_error> {
        let tx = self.conn.transaction().unwrap();

        for cmd in CHAINSTATE_INITIAL_SCHEMA {
            tx.execute_batch(cmd)?;
        }

        tx.commit()
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
            .map(|x| VRFSeed::from_proof(&x.anchored_header.proof))
    }

    fn get_miner_address(&self, id_bhh: &StacksBlockId) -> Option<StacksAddress> {
        get_miner_info(&self.conn, id_bhh).map(|x| x.address)
    }
}

fn get_stacks_header_info(conn: &Connection, id_bhh: &StacksBlockId) -> Option<StacksHeaderInfo> {
    dbg!(id_bhh);
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

const CHAINSTATE_INITIAL_SCHEMA: &'static [&'static str] = &[
    "PRAGMA foreign_keys = ON;",
    r#"
    -- Anchored stacks block headers
    CREATE TABLE block_headers(
        version INTEGER NOT NULL,
        total_burn TEXT NOT NULL,       -- converted to/from u64
        total_work TEXT NOT NULL,       -- converted to/from u64
        proof TEXT NOT NULL,
        parent_block TEXT NOT NULL,             -- hash of parent Stacks block
        parent_microblock TEXT NOT NULL,
        parent_microblock_sequence INTEGER NOT NULL,
        tx_merkle_root TEXT NOT NULL,
        state_index_root TEXT NOT NULL,
        microblock_pubkey_hash TEXT NOT NULL,
        
        block_hash TEXT NOT NULL,                   -- NOTE: this is *not* unique, since two burn chain forks can commit to the same Stacks block.
        index_block_hash TEXT UNIQUE NOT NULL,      -- NOTE: this is the hash of the block hash and consensus hash of the burn block that selected it, 
                                                    -- and is guaranteed to be globally unique (across all Stacks forks and across all PoX forks).
                                                    -- index_block_hash is the block hash fed into the MARF index.

        -- internal use only
        block_height INTEGER NOT NULL,
        index_root TEXT NOT NULL,                    -- root hash of the internal, not-consensus-critical MARF that allows us to track chainstate /fork metadata
        consensus_hash TEXT UNIQUE NOT NULL,         -- all consensus hashes are guaranteed to be unique
        burn_header_hash TEXT NOT NULL,              -- burn header hash corresponding to the consensus hash (NOT guaranteed to be unique, since we can have 2+ blocks per burn block if there's a PoX fork)
        burn_header_height INT NOT NULL,             -- height of the burnchain block header that generated this consensus hash
        burn_header_timestamp INT NOT NULL,          -- timestamp from burnchain block header that generated this consensus hash
        parent_block_id TEXT NOT NULL,               -- NOTE: this is the parent index_block_hash

        cost TEXT NOT NULL,
        block_size TEXT NOT NULL,       -- converted to/from u64

        PRIMARY KEY(consensus_hash,block_hash)
    );"#,
    "CREATE INDEX index_block_hash_to_primary_key ON block_headers(index_block_hash,consensus_hash,block_hash);",
    "CREATE INDEX block_headers_hash_index ON block_headers(block_hash,block_height);",
    "CREATE INDEX block_index_hash_index ON block_headers(index_block_hash,consensus_hash,block_hash);",
    r#"
    -- scheduled payments
    -- no designated primary key since there can be duplicate entries
    CREATE TABLE payments(
        address TEXT NOT NULL,              -- miner that produced this block and microblock stream
        block_hash TEXT NOT NULL,
        consensus_hash TEXT NOT NULL,
        parent_block_hash TEXT NOT NULL,
        parent_consensus_hash TEXT NOT NULL,
        coinbase TEXT NOT NULL,             -- encodes u128
        tx_fees_anchored TEXT NOT NULL,     -- encodes u128
        tx_fees_streamed TEXT NOT NULL,     -- encodes u128
        stx_burns TEXT NOT NULL,            -- encodes u128
        burnchain_commit_burn INT NOT NULL,
        burnchain_sortition_burn INT NOT NULL,
        miner INT NOT NULL,
        
        -- internal use
        stacks_block_height INTEGER NOT NULL,
        index_block_hash TEXT NOT NULL,     -- NOTE: can't enforce UNIQUE here, because there will be multiple entries per block
        vtxindex INT NOT NULL               -- user burn support vtxindex
    );"#,
    r#"
    -- users who supported miners
    CREATE TABLE user_supporters(
        address TEXT NOT NULL,
        support_burn INT NOT NULL,
        block_hash TEXT NOT NULL,
        consensus_hash TEXT NOT NULL,


        PRIMARY KEY(address,block_hash,consensus_hash)
    );"#,
    r#"
    CREATE TABLE db_config(
        version TEXT NOT NULL,
        mainnet INTEGER NOT NULL,
        chain_id INTEGER NOT NULL
    );"#,
    r#"
    -- Staging microblocks -- preprocessed microblocks queued up for subsequent processing and inclusion in the chunk store.
    CREATE TABLE staging_microblocks(anchored_block_hash TEXT NOT NULL,     -- this is the hash of the parent anchored block
                                     consensus_hash TEXT NOT NULL,          -- this is the hash of the burn chain block that holds the parent anchored block's block-commit
                                     index_block_hash TEXT NOT NULL,        -- this is the anchored block's index hash
                                     microblock_hash TEXT NOT NULL,
                                     parent_hash TEXT NOT NULL,             -- previous microblock
                                     index_microblock_hash TEXT NOT NULL,   -- this is the hash of consensus_hash and microblock_hash
                                     sequence INT NOT NULL,
                                     processed INT NOT NULL,
                                     orphaned INT NOT NULL,
                                     PRIMARY KEY(anchored_block_hash,consensus_hash,microblock_hash)
    );"#,
    "CREATE INDEX staging_microblocks_index_hash ON staging_microblocks(index_block_hash);",
    r#"
    -- Staging microblocks data
    CREATE TABLE staging_microblocks_data(block_hash TEXT NOT NULL,
                                          block_data BLOB NOT NULL,
                                          PRIMARY KEY(block_hash)
    );"#,
    r#"
    -- Invalidated staging microblocks data
    CREATE TABLE invalidated_microblocks_data(block_hash TEXT NOT NULL,
                                              block_data BLOB NOT NULL,
                                              PRIMARY KEY(block_hash)
    );"#,
    r#"
    -- Staging blocks -- preprocessed blocks queued up for subsequent processing and inclusion in the chunk store.
    CREATE TABLE staging_blocks(anchored_block_hash TEXT NOT NULL,
                                parent_anchored_block_hash TEXT NOT NULL,
                                consensus_hash TEXT NOT NULL,
                                -- parent_consensus_hash is the consensus hash of the parent sortition of the sortition that chose this block
                                parent_consensus_hash TEXT NOT NULL,
                                parent_microblock_hash TEXT NOT NULL,
                                parent_microblock_seq INT NOT NULL,
                                microblock_pubkey_hash TEXT NOT NULL,
                                height INT NOT NULL,
                                attachable INT NOT NULL,            -- set to 1 if this block's parent is processed; 0 if not
                                orphaned INT NOT NULL,              -- set to 1 if this block can never be attached
                                processed INT NOT NULL,
                                commit_burn INT NOT NULL,
                                sortition_burn INT NOT NULL,
                                index_block_hash TEXT NOT NULL,           -- used internally; hash of consensus hash and block header
                                download_time INT NOT NULL,               -- how long the block was in-flight
                                arrival_time INT NOT NULL,                -- when this block was stored
                                processed_time INT NOT NULL,              -- when this block was processed
                                PRIMARY KEY(anchored_block_hash,consensus_hash)
    );"#,
    "CREATE INDEX processed_stacks_blocks ON staging_blocks(processed,anchored_block_hash,consensus_hash);",
    "CREATE INDEX orphaned_stacks_blocks ON staging_blocks(orphaned,anchored_block_hash,consensus_hash);",
    "CREATE INDEX parent_blocks ON staging_blocks(parent_anchored_block_hash);",
    "CREATE INDEX parent_consensus_hashes ON staging_blocks(parent_consensus_hash);",
    "CREATE INDEX index_block_hashes ON staging_blocks(index_block_hash);",
    r#"
    -- users who burned in support of a block
    CREATE TABLE staging_user_burn_support(anchored_block_hash TEXT NOT NULL,
                                           consensus_hash TEXT NOT NULL,
                                           address TEXT NOT NULL,
                                           burn_amount INT NOT NULL,
                                           vtxindex INT NOT NULL
    );"#,
    r#"
    CREATE TABLE transactions(
        id INTEGER PRIMARY KEY,
        txid TEXT NOT NULL,
        index_block_hash TEXT NOT NULL,
        tx_hex TEXT NOT NULL,
        result TEXT NOT NULL,
        UNIQUE (txid,index_block_hash)
    );"#,
    "CREATE INDEX txid_tx_index ON transactions(txid);",
    "CREATE INDEX index_block_hash_tx_index ON transactions(index_block_hash);",
];
