use colored::*;
use radix_engine::ledger::*;
use radix_engine::utils::*;
use scrypto::types::*;

use crate::utils::*;

pub fn dump_package<T: Ledger>(address: Address, ledger: &T) {
    let package = ledger.get_package(address);
    match package {
        Some(b) => {
            println!("{}: {}", "Package".green().bold(), address.to_string());
            println!("{}: {} bytes", "Code size".green().bold(), b.code().len());
        }
        None => {
            println!("{}", "Package not found".red());
        }
    }
}

pub fn dump_component<T: Ledger>(address: Address, ledger: &T) {
    let component = ledger.get_component(address);
    match component {
        Some(c) => {
            println!("{}: {}", "Component".green().bold(), address.to_string());

            println!("{}: {:?}", "Blueprint".green().bold(), c.blueprint());
            let mut vaults = vec![];
            println!(
                "{}: {}",
                "State".green().bold(),
                format_sbor_with_ledger(c.state(), ledger, &mut vaults).unwrap()
            );

            println!("{}:", "Resources".green().bold());
            for (last, vid) in vaults.iter().identify_last() {
                let vault = ledger.get_vault(*vid).unwrap();
                println!(
                    "{} {{ amount: {}, resource_def: {} }}",
                    list_item_prefix(last),
                    vault.amount(),
                    vault.resource_def(),
                );
            }
        }
        None => {
            println!("{}", "Component not found".red());
        }
    }
}

pub fn dump_resource_def<T: Ledger>(address: Address, ledger: &T) {
    let resource_def = ledger.get_resource_def(address);
    match resource_def {
        Some(r) => {
            for (k, v) in r.metadata {
                println!("{}: {}", k.green().bold(), v);
            }
            println!("{}: {:?}", "Minter".green().bold(), r.minter);
            println!("{}: {:?}", "supply".green().bold(), r.supply);
        }
        None => {
            println!("{}", "Resource definition not found".red());
        }
    }
}
