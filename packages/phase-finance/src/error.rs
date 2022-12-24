use cosmwasm_std::StdError;
use cw_utils::{Expiration, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No balance")]
    NoBalance {},

    #[error("DCA swap not allowed yet, next swap will be executable at {next_swap_event_time:?}")]
    DcaSwapNotAllowedYet { next_swap_event_time: u64 },

    #[error("DCA strategy is paused")]
    DcaPaused,

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
