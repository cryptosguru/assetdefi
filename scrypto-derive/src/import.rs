use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::*;

use sbor::describe as des;
use scrypto_abi as abi;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_import(input: TokenStream) -> TokenStream {
    trace!("Started processing import macro");

    let content = parse2::<LitStr>(input)
        .expect("Unable to parse input")
        .value();
    let blueprint: abi::Blueprint =
        serde_json::from_str(content.as_str()).expect("Unable to parse ABI");
    trace!("Parsed ABI: {:?}", blueprint);

    let package = blueprint.package;
    let name = blueprint.name;
    let ident = format_ident!("{}", name);
    trace!("Blueprint name: {}", name);

    let mut structs: Vec<Item> = vec![];

    let mut functions = Vec::<ItemFn>::new();
    for function in &blueprint.functions {
        trace!("Processing function: {:?}", function);

        let func_name = &function.name;
        let func_indent = format_ident!("{}", func_name);
        let mut func_types = Vec::<Type>::new();
        let mut func_args = Vec::<Ident>::new();

        for (i, input) in function.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_structs) = get_native_type(input);
            func_args.push(parse_quote! { #ident });
            func_types.push(parse_quote! { #new_type });
            structs.extend(new_structs);
        }
        let (func_output, new_structs) = get_native_type(&function.output);
        structs.extend(new_structs);

        functions.push(parse_quote! {
            pub fn #func_indent(#(#func_args: #func_types),*) -> #func_output {
                let package = ::scrypto::utils::unwrap_light(
                    ::scrypto::types::Address::from_str(#package)
                );
                let rtn = ::scrypto::core::call_function(
                    package,
                    #name,
                    #func_name,
                    ::scrypto::args!(#(#func_args),*)
                );
                ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
            }
        });
    }

    let mut methods = Vec::<ItemFn>::new();
    for method in &blueprint.methods {
        trace!("Processing method: {:?}", method);

        let method_name = &method.name;
        let method_indent = format_ident!("{}", method_name);
        let mut method_types = Vec::<Type>::new();
        let mut method_args = Vec::<Ident>::new();

        for (i, input) in method.inputs.iter().enumerate() {
            let ident = format_ident!("arg{}", i);
            let (new_type, new_structs) = get_native_type(input);
            method_args.push(parse_quote! { #ident });
            method_types.push(parse_quote! { #new_type });
            structs.extend(new_structs);
        }
        let (method_output, new_structs) = get_native_type(&method.output);
        structs.extend(new_structs);

        let m = parse_quote! {
            pub fn #method_indent(&self #(, #method_args: #method_types)*) -> #method_output {
                let rtn = ::scrypto::core::call_method(
                    self.address,
                    #method_name,
                    ::scrypto::args!(#(#method_args),*)
                );
                ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
            }
        };
        methods.push(m);
    }

    let output = quote! {
        #(#structs)*

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
        pub struct #ident {
            address: ::scrypto::types::Address,
        }

        impl #ident {
            #(#functions)*

            #(#methods)*
        }

        impl From<::scrypto::types::Address> for #ident {
            fn from(address: ::scrypto::types::Address) -> Self {
                Self {
                    address
                }
            }
        }

        impl From<#ident> for ::scrypto::types::Address {
            fn from(a: #ident) -> ::scrypto::types::Address {
                a.address
            }
        }

        impl From<::scrypto::core::Component> for #ident {
            fn from(component: ::scrypto::core::Component) -> Self {
                Self {
                    address: component.into()
                }
            }
        }

        impl From<#ident> for ::scrypto::core::Component {
            fn from(a: #ident) -> ::scrypto::core::Component {
                a.address.into()
            }
        }
    };
    trace!("Finished processing import macro");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("import!", &output);

    output
}

fn get_native_type(ty: &des::Type) -> (Type, Vec<Item>) {
    let mut structs = Vec::<Item>::new();

    let t: Type = match ty {
        // primitive types
        des::Type::Unit => parse_quote! { () },
        des::Type::Bool => parse_quote! { bool },
        des::Type::I8 => parse_quote! { i8 },
        des::Type::I16 => parse_quote! { i16 },
        des::Type::I32 => parse_quote! { i32 },
        des::Type::I64 => parse_quote! { i64 },
        des::Type::I128 => parse_quote! { i128 },
        des::Type::U8 => parse_quote! { u8 },
        des::Type::U16 => parse_quote! { u16 },
        des::Type::U32 => parse_quote! { u32 },
        des::Type::U64 => parse_quote! { u64 },
        des::Type::U128 => parse_quote! { u128 },
        des::Type::String => parse_quote! { String },
        // struct & enum
        des::Type::Struct { name, fields } => {
            let ident = format_ident!("{}", name);

            match fields {
                des::Fields::Named { named } => {
                    let names: Vec<Ident> =
                        named.iter().map(|k| format_ident!("{}", k.0)).collect();
                    let mut types: Vec<Type> = vec![];
                    for (_, v) in named {
                        let (new_type, new_structs) = get_native_type(v);
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident {
                            #( pub #names : #types, )*
                        }
                    });
                }
                des::Fields::Unnamed { unnamed } => {
                    let mut types: Vec<Type> = vec![];
                    for v in unnamed {
                        let (new_type, new_structs) = get_native_type(v);
                        types.push(new_type);
                        structs.extend(new_structs);
                    }
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident (
                            #( pub #types ),*
                        )
                    });
                }
                des::Fields::Unit => {
                    structs.push(parse_quote! {
                        #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
                        pub struct #ident;
                    });
                }
            }

            parse_quote! { #ident }
        }
        des::Type::Enum { name, variants } => {
            let ident = format_ident!("{}", name);
            let mut native_variants = Vec::<Variant>::new();

            for variant in variants {
                let v_ident = format_ident!("{}", variant.name);

                match &variant.fields {
                    des::Fields::Named { named } => {
                        let mut names: Vec<Ident> = vec![];
                        let mut types: Vec<Type> = vec![];
                        for (n, v) in named {
                            names.push(format_ident!("{}", n));
                            let (new_type, new_structs) = get_native_type(v);
                            types.push(new_type);
                            structs.extend(new_structs);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident {
                                #(#names: #types),*
                            }
                        });
                    }
                    des::Fields::Unnamed { unnamed } => {
                        let mut types: Vec<Type> = vec![];
                        for v in unnamed {
                            let (new_type, new_structs) = get_native_type(v);
                            types.push(new_type);
                            structs.extend(new_structs);
                        }
                        native_variants.push(parse_quote! {
                            #v_ident ( #(#types),* )
                        });
                    }
                    des::Fields::Unit => {
                        native_variants.push(parse_quote! {
                            #v_ident
                        });
                    }
                };
            }

            structs.push(parse_quote! {
                #[derive(Debug, ::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                pub enum #ident {
                    #( #native_variants ),*
                }
            });

            parse_quote! { #ident }
        }
        // composite types
        des::Type::Option { value } => {
            let (new_type, new_structs) = get_native_type(value);
            structs.extend(new_structs);

            parse_quote! { Option<#new_type> }
        }
        des::Type::Box { value } => {
            let (new_type, new_structs) = get_native_type(value);
            structs.extend(new_structs);

            parse_quote! { Box<#new_type> }
        }
        des::Type::Tuple { elements } => {
            let mut types: Vec<Type> = vec![];

            for element in elements {
                let (new_type, new_structs) = get_native_type(element);
                types.push(new_type);
                structs.extend(new_structs);
            }

            parse_quote! { ( #(#types),* ) }
        }
        des::Type::Array { element, length } => {
            let (new_type, new_structs) = get_native_type(element);
            structs.extend(new_structs);

            let n = *length as usize;
            parse_quote! { [#new_type; #n] }
        }
        des::Type::Result { okay, error } => {
            let (okay_type, new_structs) = get_native_type(okay);
            structs.extend(new_structs);
            let (error_type, new_structs) = get_native_type(error);
            structs.extend(new_structs);

            parse_quote! { Result<#okay_type, #error_type> }
        }
        // collection
        des::Type::Vec { element } => {
            let (new_type, new_structs) = get_native_type(element);
            structs.extend(new_structs);

            parse_quote! { Vec<#new_type> }
        }
        des::Type::TreeSet { element } => {
            let (new_type, new_structs) = get_native_type(element);
            structs.extend(new_structs);

            parse_quote! { BTreeSet<#new_type> }
        }
        des::Type::TreeMap { key, value } => {
            let (key_type, new_structs) = get_native_type(key);
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value);
            structs.extend(new_structs);

            parse_quote! { BTreeMap<#key_type, #value_type> }
        }
        des::Type::HashSet { element } => {
            let (new_type, new_structs) = get_native_type(element);
            structs.extend(new_structs);

            parse_quote! { HashSet<#new_type> }
        }
        des::Type::HashMap { key, value } => {
            let (key_type, new_structs) = get_native_type(key);
            structs.extend(new_structs);
            let (value_type, new_structs) = get_native_type(value);
            structs.extend(new_structs);

            parse_quote! { HashMap<#key_type, #value_type> }
        }
        des::Type::Custom { name } => {
            if name.starts_with("scrypto::") {
                let ty: Type = parse_str(&format!("::{}", name)).unwrap();
                parse_quote! { #ty }
            } else {
                panic!("Invalid custom type: {}", name)
            }
        }
    };

    (t, structs)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_import_empty() {
        let input = TokenStream::from_str(
            r###"
                r#"
                {
                    "package": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
                    "name": "Simple",
                    "functions": [
                        {
                            "name": "new",
                            "inputs": [],
                            "output": {
                                "type": "Custom",
                                "name": "scrypto::core::Component"
                            }
                        }
                    ],
                    "methods": [
                        {
                            "name": "free_token",
                            "mutability": "Mutable",
                            "inputs": [
                            ],
                            "output": {
                                "type": "Custom",
                                "name": "scrypto::resource::Bucket"
                            }
                        }
                    ]
                }
                "#
            "###,
        )
        .unwrap();
        let output = handle_import(input);

        assert_code_eq(
            output,
            quote! {
                #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode)]
                pub struct Simple {
                    address: ::scrypto::types::Address,
                }
                impl Simple {
                    pub fn new() -> ::scrypto::core::Component {
                        let package = ::scrypto::utils::unwrap_light(::scrypto::types::Address::from_str(
                            "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7"
                        ));
                        let rtn = ::scrypto::core::call_function(package, "Simple", "new", ::scrypto::args!());
                        ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
                    }
                    pub fn free_token(&self) -> ::scrypto::resource::Bucket {
                        let rtn = ::scrypto::core::call_method(self.address, "free_token", ::scrypto::args!());
                        ::scrypto::utils::unwrap_light(::scrypto::buffer::scrypto_decode(&rtn))
                    }
                }
                impl From<::scrypto::types::Address> for Simple {
                    fn from(address: ::scrypto::types::Address) -> Self {
                        Self { address }
                    }
                }
                impl From<Simple> for ::scrypto::types::Address {
                    fn from(a: Simple) -> ::scrypto::types::Address {
                        a.address
                    }
                }
                impl From<::scrypto::core::Component> for Simple {
                    fn from(component: ::scrypto::core::Component) -> Self {
                        Self {
                            address: component.into()
                        }
                    }
                }
                impl From<Simple> for ::scrypto::core::Component {
                    fn from(a: Simple) -> ::scrypto::core::Component {
                        a.address.into()
                    }
                }
            },
        );
    }
}
