use super::structs::pool::Pool;
use ethers::types::Transaction;
use crate::utils::types::structs::token::Token;

// New pair event from the pair oracle
#[derive(Debug, Clone)]
pub enum NewPoolEvent {
    NewPool {
        pool: Pool,
        // tx: Transaction,
    },
}


// New mempool event from the mempool stream
#[derive(Debug, Clone)]
pub enum MemPoolEvent {
    NewTx {
        tx: Transaction,
    },
}

#[derive(Debug, Clone)]
pub enum NewTokenEvent {
    NewToken {
        token: Token,
        // tx: Transaction,
    },
}
