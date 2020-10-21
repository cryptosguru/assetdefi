mod ast;
mod auth;
mod blueprint;
mod import;
mod utils;

use proc_macro::TokenStream;

/// Defines a blueprint.
///
/// The `blueprint!` macro is a convenient way to define a new blueprint. It takes
/// two arguments:
/// - A `struct` which defines the structure
/// - A `impl` which defines the implementation.
///
/// This macro will derive the dispatcher method responsible for handling invocation
/// according to Scrypto ABI.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// blueprint! {
///     struct Counter {
///         count: u32
///     }
///
///     impl Counter {
///         pub fn new() -> Component {
///             Self {
///                 count: 0
///             }.instantiate()
///         }
///
///         pub fn get_and_incr(&mut self) -> u32 {
///             let n = self.count;
///             self.count += 1;
///             n
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn blueprint(input: TokenStream) -> TokenStream {
    let output = blueprint::handle_blueprint(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

/// Imports a blueprint from its ABI.
///
/// This macro will generate stubs for accessing the blueprint according to
/// its ABI specification.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// import! {
/// r#"
/// {
///     "package": "01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876",
///     "name": "GumballMachine",
///     "functions": [
///         {
///             "name": "new",
///             "inputs": [],
///             "output": {
///                 "type": "Custom",
///                 "name": "scrypto::types::Address"
///             }
///         }
///     ],
///     "methods": [
///         {
///             "name": "get_gumball",
///             "mutability": "Mutable",
///             "inputs": [
///                 {
///                     "type": "Custom",
///                     "name": "scrypto::resource::Bucket"
///                 }
///             ],
///             "output": {
///                 "type": "Custom",
///                 "name": "scrypto::resource::Bucket"
///             }
///         }
///     ]
/// }
/// "#
/// }
/// ```
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let output = import::handle_import(proc_macro2::TokenStream::from(input));
    TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn auth(attr: TokenStream, item: TokenStream) -> TokenStream {
    auth::handle_auth(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
