use cosmwasm_std::{Deps, MessageInfo, Uint128};

use crate::{croncat_helpers::calculate_croncat_funding, error::ContractError};

// require correct funding for dca + croncat if enabled
pub fn validate_funding(
    deps: Deps,
    info: &MessageInfo,
    amount_per_trade: Uint128,
    num_trades: Uint128,
    use_croncat: bool,
) -> Result<(), ContractError> {
    // check that amount deposited is correct for dca params
    let required_base_deposit = amount_per_trade * num_trades;

    // check that amount of source coins is equal to 1
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError {
            val: "amount of source coins is not equal to 1".to_string(),
        });
    }

    if use_croncat {
        // get croncat configurated amount
        let croncat_fee = calculate_croncat_funding(deps, num_trades)?;

        if required_base_deposit + croncat_fee[0].amount != info.funds[0].amount {
            return Err(ContractError::CustomError {
                val: "amount deposited does not match amount per trade and num trades plus croncat_fee".to_string(),
            });
        }
    } else {
        if required_base_deposit != info.funds[0].amount {
            return Err(ContractError::CustomError {
                val: "amount deposited does not match amount per trade and num trades".to_string(),
            });
        }
    }

    Ok(())
}
