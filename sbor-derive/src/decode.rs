use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

use crate::utils::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_decode(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_decode() starts");

    let DeriveInput { ident, data, .. } = parse2(input)?;
    trace!("Decoding: {}", ident);

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // ns: not skipped
                let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                let ns_n = Index::from(ns.len());
                let ns_ids = ns.iter().map(|f| &f.ident);
                let ns_types = ns.iter().map(|f| &f.ty);
                let s: Vec<&Field> = named.iter().filter(|f| is_skipped(f)).collect();
                let s_ids = s.iter().map(|f| &f.ident);
                let s_types = s.iter().map(|f| &f.ty);
                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};
                            decoder.check_type(::sbor::type_id::TYPE_FIELDS_NAMED)?;
                            decoder.check_len(#ns_n)?;
                            Ok(Self {
                                #(#ns_ids: <#ns_types>::decode(decoder)?,)*
                                #(#s_ids: <#s_types>::default()),*
                            })
                        }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut all_exprs = Vec::<Expr>::new();
                for f in &unnamed {
                    let ty = &f.ty;
                    if is_skipped(f) {
                        all_exprs.push(parse_quote! {<#ty>::default()})
                    } else {
                        all_exprs.push(parse_quote! {<#ty>::decode(decoder)?})
                    }
                }
                let ns_n = Index::from(unnamed.iter().filter(|f| !is_skipped(f)).count());
                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{self, Decode};
                            decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNNAMED)?;
                            decoder.check_len(#ns_n)?;
                            Ok(Self (
                                #(#all_exprs,)*
                            ))
                        }
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    impl ::sbor::Decode for #ident {
                        fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                            decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNIT)?;
                            Ok(Self {})
                        }
                    }
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let match_arms = variants.iter().enumerate().map(|(i, v)| {
                let v_id = &v.ident;
                let v_ith = i as u8;
                match &v.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let ns: Vec<&Field> = named.iter().filter(|f| !is_skipped(f)).collect();
                        let ns_n = Index::from(ns.len());
                        let ns_ids = ns.iter().map(|f| &f.ident);
                        let ns_types = ns.iter().map(|f| &f.ty);
                        let s: Vec<&Field> = named.iter().filter(|f| is_skipped(f)).collect();
                        let s_ids = s.iter().map(|f| &f.ident);
                        let s_types = s.iter().map(|f| &f.ty);
                        quote! {
                            #v_ith => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_NAMED)?;
                                decoder.check_len(#ns_n)?;

                                Ok(Self::#v_id {
                                    #(#ns_ids: <#ns_types>::decode(decoder)?,)*
                                    #(#s_ids: <#s_types>::default(),)*
                                })
                            }
                        }
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let mut all_exprs = Vec::<Expr>::new();
                        for f in unnamed {
                            let ty = &f.ty;
                            if is_skipped(f) {
                                all_exprs.push(parse_quote! {<#ty>::default()})
                            } else {
                                all_exprs.push(parse_quote! {<#ty>::decode(decoder)?})
                            }
                        }
                        let ns_n = Index::from(unnamed.iter().filter(|f| !is_skipped(f)).count());
                        quote! {
                            #v_ith => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNNAMED)?;
                                decoder.check_len(#ns_n)?;

                                Ok(Self::#v_id (
                                    #(#all_exprs),*
                                ))
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #v_ith => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNIT)?;
                                Ok(Self::#v_id)
                            }
                        }
                    }
                }
            });

            quote! {
                impl ::sbor::Decode for #ident {
                    #[inline]
                    fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};

                        let index = decoder.read_u8()?;
                        match index {
                            #(#match_arms,)*
                            _ => Err(::sbor::DecodeError::InvalidIndex(index))
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new(Span::call_site(), "Union is not supported!"));
        }
    };
    trace!("handle_decode() finishes");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("Decode", &output);

    Ok(output)
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
    fn test_decode_struct() {
        let input = TokenStream::from_str("struct Test {a: u32}").unwrap();
        let output = handle_decode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Decode for Test {
                    fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        decoder.check_type(::sbor::type_id::TYPE_FIELDS_NAMED)?;
                        decoder.check_len(1)?;
                        Ok(Self {
                            a: <u32>::decode(decoder)?,
                        })
                    }
                }
            },
        );
    }

    #[test]
    fn test_decode_enum() {
        let input = TokenStream::from_str("enum Test {A, B (u32), C {x: u8}}").unwrap();
        let output = handle_decode(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::sbor::Decode for Test {
                    #[inline]
                    fn decode_value(decoder: &mut ::sbor::Decoder) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{self, Decode};
                        let index = decoder.read_u8()?;
                        match index {
                            0u8 => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNIT)?;
                                Ok(Self::A)
                            },
                            1u8 => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_UNNAMED)?;
                                decoder.check_len(1)?;
                                Ok(Self::B(<u32>::decode(decoder)?))
                            },
                            2u8 => {
                                decoder.check_type(::sbor::type_id::TYPE_FIELDS_NAMED)?;
                                decoder.check_len(1)?;
                                Ok(Self::C {
                                    x: <u8>::decode(decoder)?,
                                })
                            },
                            _ => Err(::sbor::DecodeError::InvalidIndex(index))
                        }
                    }
                }
            },
        );
    }
}
