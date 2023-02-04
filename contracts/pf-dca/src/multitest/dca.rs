use crate::multitest::common::*;
use crate::multitest::suite::*;

#[test]
fn not_executor_cannot_perform_dca() {
    let mut suite = PhaseFinanceSuite::init(None, None).unwrap();
    let err = suite.try_perform_dca(&suite.user.clone()).unwrap_err();
    assert_eq!(err, DCAContractError::Unauthorized {});
}
