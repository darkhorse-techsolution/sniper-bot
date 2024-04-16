use ethers_core::abi::Abi;
use std::fs;

pub struct ABI {
    // pub bot: Abi,
    pub uniswap_v2_factory: Abi,
    // pub uniswap_v2_factory: Abi,
    pub erc20: Abi,
    pub uniswap_v2_pair: Abi
}

impl ABI {
    pub fn new() -> Self {
        let erc20_json = fs::read_to_string("src/abi/ERC20.json").unwrap();
        let uniswap_v2_factory = fs::read_to_string("src/abi/factory.json").unwrap();
        let uniswap_v2_pair = fs::read_to_string("src/abi/UniswapV2Pair.json").unwrap();

        Self {
            erc20: serde_json::from_str(&erc20_json).unwrap(),
            // bot: serde_json::from_str(&bot_json).unwrap(),
            // weth: serde_json::from_str(&weth_json).unwrap(),
            // uniswap_v2_factory: serde_json::from_str(&uniswap_v2_factory_json).unwrap(),
            uniswap_v2_factory: serde_json::from_str(&uniswap_v2_factory).unwrap(),
            uniswap_v2_pair: serde_json::from_str(&uniswap_v2_pair).unwrap()
            // v2_arb_bot: serde_json::from_str(&v2_arb_bot_json).unwrap(),
        }
    }
}

