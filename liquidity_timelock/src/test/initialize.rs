use crate::error::CombinedLiquidityTimelockError;
use crate::test::AddLiqudityTimelockTest;

#[test]
fn test_initialize_and_get_values() {
    let test = AddLiqudityTimelockTest::setup();
    let desired_release_time = 1746885472;

    test.timelock_contract.initialize(
        &test.admin,
        &test.router_contract.address,
        &desired_release_time,
    );

    let admin = test.timelock_contract.get_admin();
    assert_eq!(admin, test.admin);

    // test release time
    let release_time = test.timelock_contract.get_release_time();
    assert_eq!(release_time, desired_release_time);
}

#[test]
fn test_get_admin_not_yet_initialized() {
    let test = AddLiqudityTimelockTest::setup();
    let result = test.timelock_contract.try_get_admin();

    assert_eq!(
        result,
        Err(Ok(CombinedLiquidityTimelockError::TimelockNotInitialized))
    );
}

#[test]
fn test_get_release_time_not_yet_initialized() {
    let test = AddLiqudityTimelockTest::setup();
    let result = test.timelock_contract.try_get_release_time();

    assert_eq!(
        result,
        Err(Ok(CombinedLiquidityTimelockError::TimelockNotInitialized))
    );
}

#[test]
fn test_initialize_twice() {
    let test = AddLiqudityTimelockTest::setup();

    test.timelock_contract
        .initialize(&test.admin, &test.router_contract.address, &1746885472);

    let result_second_init = test.timelock_contract.try_initialize(
        &test.admin,
        &test.router_contract.address,
        &1746885472,
    );

    assert_eq!(
        result_second_init,
        (Err(Ok(
            CombinedLiquidityTimelockError::TimelockAlreadyInitialized
        )))
    );
}
