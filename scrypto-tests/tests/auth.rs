#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::abi;
use scrypto::buffer::*;
use scrypto::prelude::*;
use serde::Serialize;
use serde_json::{json, to_value, Value};

blueprint! {
    struct SimpleAuth {
        admin: ResourceDef,
        user: ResourceDef,
        reserves: Vault,
    }

    impl SimpleAuth {
        pub fn new(admin: ResourceDef, user: ResourceDef) -> Component {
            Self {
                admin,
                user,
                reserves: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
        }

        //#[auth(all(admin))]
        pub fn pump(&self, xrd: Bucket) {
            self.reserves.put(xrd);
        }

        //#[auth(all(user))]
        pub fn airdrop(&self) -> Bucket {
            self.reserves.take(1)
        }
    }
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_simple_auth() {
    let ptr = SimpleAuth_abi();
    let abi: (Vec<abi::Function>, Vec<abi::Method>) =
        unsafe { scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap()) };

    assert_json_eq(
        abi,
        json!([
          [
            {
              "name": "new",
              "inputs": [
                {
                  "type": "Custom",
                  "name": "scrypto::resource::ResourceDef",
                  "generics": []
                },
                {
                  "type": "Custom",
                  "name": "scrypto::resource::ResourceDef",
                  "generics": []
                }
              ],
              "output": {
                "type": "Custom",
                "name": "scrypto::core::Component",
                "generics": []
              }
            }
          ],
          [
            {
              "name": "pump",
              "mutability": "Immutable",
              "inputs": [
                {
                  "type": "Custom",
                  "name": "scrypto::resource::Bucket",
                  "generics": []
                }
              ],
              "output": {
                "type": "Unit"
              }
            },
            {
              "name": "airdrop",
              "mutability": "Immutable",
              "inputs": [],
              "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
              }
            }
          ]
        ]),
    );
}
