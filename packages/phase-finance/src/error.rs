use cosmwasm_std::StdError;
use cw_denom::DenomError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    InvalidDenom(#[from] DenomError),

    #[error("No balance")]
    NoBalance {},

    #[error("DCA swap not allowed yet, next swap will be executable at {next_swap_event_time:?}")]
    DcaSwapNotAllowedYet { next_swap_event_time: u64 },

    #[error("DCA strategy is paused")]
    DcaPaused,

    #[error("DCA strategy is not paused")]
    DcaNotPaused,

    #[error("Reached max trade limit")]
    MaxTradeLimit {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
