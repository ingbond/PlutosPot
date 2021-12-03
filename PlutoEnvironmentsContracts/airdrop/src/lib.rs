pub mod contract;
pub mod msg;
pub mod state;
mod error;
pub use crate::error::ContractError;

#[cfg(test)]
mod testing;

//#[cfg(target_arch = "wasm32")]
//cosmwasm_std::create_entry_points_with_migration!(contract);
