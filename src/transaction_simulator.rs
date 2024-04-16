use ethers::prelude::LocalWallet;
use ethers::signers::Signer;
use ethers_core::abi::RawLog;
use ethers_core::types::{Bytes, Call, Eip1559TransactionRequest, Transaction};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::utils::hex::hex;

use crate::constants::Env;
pub struct TransactionSimulator {
    tx: Transaction,
    pub sender: LocalWallet,
}

impl TransactionSimulator {
    pub fn new(tx: Transaction) -> TransactionSimulator {
        let env = Env::new();
        let sender = env
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(369_u64);
        Self {
            tx,
            sender
        }
    }

    pub fn get_method_id(&self) -> String {
        let tx = self.tx.clone();
        let input_data = tx.input.clone();

        let function_name = hex::encode(&input_data[0..4]);
        function_name
    }

    pub async fn sign_tx(&self, tx: Eip1559TransactionRequest) -> anyhow::Result<Bytes> {
        let typed = TypedTransaction::Eip1559(tx);
        let signature = self.sender.sign_transaction(&typed).await?;
        let signed = typed.rlp_signed(&signature);
        Ok(signed)
    }

    pub async fn simulate_transaction(&self) -> Result<Vec<RawLog>, Box<dyn std::error::Error>> {
        let tx = self.tx.clone();
        let call = Call {
            from: tx.from,
            to: tx.to.unwrap(),
            input: tx.input.clone(),
            value: tx.value,
            ..Default::default()
        };

        // Perform an eth_call to simulate the transaction
        // let output = client.call(&call).await?;
        println!("Simulation Output: {:?}", call);

        // Parse potential logs here if needed (logs are not directly available in simulation)
        Ok(vec![]) // Replace with actual logic to interpret logs if possible
    }
}