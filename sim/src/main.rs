use std::str::FromStr;

use alloy::{
    eips::BlockId,
    primitives::{address, Address, FixedBytes, TxKind, Uint},
    providers::Provider,
    rpc::types::serde_helpers::quantity::vec,
    sol,
    sol_types::SolInterface,
    transports::http::reqwest::Url,
};
use foundry_evm::{
    backend::{BlockchainDb, BlockchainDbMeta, SharedBackend},
    revm::{db::CacheDB, primitives::ExecutionResult, Evm},
};

struct SimulatorFactory {
    db: CacheDB<SharedBackend>,
}
const MODULE_CORE: Address = address!("8445a4caD9F5a991E668427dC96A0a6b80ca629b");
const DEFAULT_CALLER: Address = address!("6B905a32b02f6C002c18F9733e4B428F59EF86a8");
const RA : Address = address!("34505854505A4a4e898569564Fb91e17614e1969");

sol!(
        interface IERC20 {
    /// @dev Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
    event Transfer(address indexed from, address indexed to, uint256 value);

    /// @dev Emitted when the allowance of a `spender` for an `owner` is set, where `value`
    /// is the new allowance.
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /// @notice Returns the amount of tokens in existence.
    function totalSupply() external view returns (uint256);

    /// @notice Returns the amount of tokens owned by `account`.
    function balanceOf(address account) external view returns (uint256);

    /// @notice Moves `amount` tokens from the caller's account to `to`.
    function transfer(address to, uint256 amount) external returns (bool);

    /// @notice Returns the remaining number of tokens that `spender` is allowed
    /// to spend on behalf of `owner`
    function allowance(address owner, address spender) external view returns (uint256);

    /// @notice Sets `amount` as the allowance of `spender` over the caller's tokens.
    /// @dev Be aware of front-running risks: https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
    function approve(address spender, uint256 amount) external returns (bool);

    /// @notice Moves `amount` tokens from `from` to `to` using the allowance mechanism.
    /// `amount` is then deducted from the caller's allowance.
    function transferFrom(address from, address to, uint256 amount) external returns (bool);

    /// @notice Returns the name of the token.
    function name() external view returns (string memory);

    /// @notice Returns the symbol of the token.
    function symbol() external view returns (string memory);

    /// @notice Returns the decimals places of the token.
    function decimals() external view returns (uint8);
}

    );

sol!(
interface IVault{
        type Id is bytes32;

    /**
     * @notice Deposit a wrapped asset into a given vault
     * @param id The Module id that is used to reference both psm and lv of a given pair
     * @param amount The amount of the redemption asset(ra) deposited
     */
    function depositLv(Id id, uint256 amount, uint256 raTolerance, uint256 ctTolerance)
    external
    returns (uint256 received);
}
);

impl SimulatorFactory {
    fn create_vm(&self) -> Evm<'_, (), CacheDB<SharedBackend>> {
        let context = foundry_evm::revm::Context::new_with_db(self.db.clone());
        Evm::builder().with_db(self.db.clone()).build()
    }
}

#[tokio::main]
async fn main() {
    let url = "https://sepolia.drpc.org";
    let url = Url::parse(url).unwrap();

    let transport = alloy::providers::builder().on_http(url.to_owned());
    let db = BlockchainDb::new(
        BlockchainDbMeta::new(Default::default(), url.to_string()),
        Some("/tmp/evm.db".try_into().unwrap()),
    );
    let block_number = transport.get_block_number().await.unwrap();

    let backend =
        SharedBackend::spawn_backend_thread(transport, db, Some(BlockId::number(block_number)));
    let db = CacheDB::new(backend);
    let factory = SimulatorFactory { db };
    let mut vm = factory.create_vm();

    vm = vm.modify().modify_tx_env(populate_tx_approve).build();
    let result = vm.transact_commit().unwrap();
    match result {
        ExecutionResult::Success { output, .. } => {
            println!("approval successful, output: {:?}", output);
        }
        ExecutionResult::Revert { gas_used, output } => {
            println!("approval reverted: {:?}", output);
        }
        ExecutionResult::Halt { reason, .. } => {
            println!("approval error: {:?}", reason);
        }
    }

    vm = vm.modify().modify_tx_env(populate_tx_deposit_lv).build();
    let result = vm.transact_commit().unwrap();
    match result {
        ExecutionResult::Success { output, .. } => {
            println!("deposit successful, output: {:?}", output);
        }
        ExecutionResult::Revert { gas_used, output } => {
            println!("deposit reverted: {:?}", output);
        }
        ExecutionResult::Halt { reason, .. } => {
            println!("deposit error: {:?}", reason);
        }
    }
}

fn populate_tx_approve(tx: &mut foundry_evm::revm::primitives::TxEnv) {
    let eth_1 = Uint::from_str("1000000000000000000").unwrap();
    let data = IERC20::IERC20Calls::approve(IERC20::approveCall {
        spender: MODULE_CORE,
        amount: eth_1,
    });

    tx.data = data.abi_encode().into();
    tx.caller = DEFAULT_CALLER;
    tx.transact_to = TxKind::Call(RA);
}

fn populate_tx_deposit_lv(tx: &mut foundry_evm::revm::primitives::TxEnv) {
    let _id = FixedBytes::<32>::from_str(
        "0x42d2e90dba3fee84441d158a2876f22245344966c9f7ed67e5ed5e1274e0efa8",
    )
    .unwrap();
    
    let data = IVault::depositLvCall {
        amount: Uint::from_str("1000000000000000000").unwrap(),
        ctTolerance: Uint::from_str("0").unwrap(),
        raTolerance: Uint::from_str("0").unwrap(),
        id: _id,
    };

    tx.data = IVault::IVaultCalls::depositLv(data).abi_encode().into();
    tx.caller = DEFAULT_CALLER;
    tx.transact_to = TxKind::Call(MODULE_CORE);
}
