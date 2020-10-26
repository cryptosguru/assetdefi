use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;
use std::process::Command;

fn compile() {
    Command::new("cargo")
        .current_dir("./tests/everything")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
}

#[test]
fn test_package() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "PackageTest",
            "publish_package",
            vec![],
            Some(account),
        )
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, true);
    assert!(receipt1.success);
}

#[test]
fn test_context() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(package, "ContextTest", "query", vec![], Some(account))
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, true);
    assert!(receipt1.success);
}

#[test]
fn test_component() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    // Create component
    let transaction1 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            vec![],
            Some(account),
        )
        .build()
        .unwrap();
    let receipt1 = executor.run(transaction1, true);
    assert!(receipt1.success);

    // Find the component address from receipt
    let component = receipt1.component(0).unwrap();

    // Call functions & methods
    let transaction2 = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ComponentTest",
            "get_component_blueprint",
            vec![component.to_string()],
            Some(account),
        )
        .call_method(component, "get_component_state", vec![], Some(account))
        .call_method(component, "put_component_state", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt2 = executor.run(transaction2, true);
    assert!(receipt2.success);
}

#[test]
fn test_lazy_map() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "LazyMapTest",
            "test_lazy_map",
            vec![],
            Some(account),
        )
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_resource_def() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "ResourceTest",
            "create_mutable",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "ResourceTest",
            "create_fixed",
            vec![],
            Some(account),
        )
        .call_function(package, "ResourceTest", "query", vec![], Some(account))
        .call_function(package, "ResourceTest", "burn", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_bucket() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BucketTest", "combine", vec![], Some(account))
        .call_function(package, "BucketTest", "split", vec![], Some(account))
        .call_function(package, "BucketTest", "borrow", vec![], Some(account))
        .call_function(package, "BucketTest", "query", vec![], Some(account))
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}

#[test]
fn test_move_resource() {
    compile();
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);
    let account = executor.create_account();
    let package = executor.publish_package(include_code!("./everything"));

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "MoveTest", "move_bucket", vec![], Some(account))
        .call_function(
            package,
            "MoveTest",
            "move_bucket_ref",
            vec![],
            Some(account),
        )
        .deposit_all(account)
        .build()
        .unwrap();
    let receipt = executor.run(transaction, true);
    assert!(receipt.success);
}
