use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_create_additional_admin() {
    // Set up environment.
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(include_code!());

    // Test the `new` function.
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "FlatAdmin", "new", vec!["test".to_string()], None)
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt1 = executor.run(transaction1, false).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.success);

    // Test the `create_additional_admin` method.
    let flat_admin = receipt1.component(0).unwrap();
    let admin_badge = receipt1.resource_def(1).unwrap();
    let transaction2 = TransactionBuilder::new(&executor)
        .call_method(
            flat_admin,
            "create_additional_admin",
            vec![format!("1,{}", admin_badge)],
            Some(account),
        )
        .drop_all_bucket_refs()
        .deposit_all_buckets(account)
        .build(vec![key])
        .unwrap();
    let receipt2 = executor.run(transaction2, false).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.success);
}
