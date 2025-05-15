pub mod geyser;

pub mod solana {
    pub mod storage {
        pub mod confirmed_block {
            include!("solana.storage.confirmed_block.rs");
        }
    }
}
