use std::sync::RwLock;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Arm, AttrStyle, Attribute, Expr, ExprArray, ExprMatch, Field, Fields, FieldsUnnamed, GenericParam, Generics, Item, ItemEnum, ItemMod, Lifetime, LifetimeDef, Pat, PatLit, Path, PathArguments, PathSegment, Type, Variant, VisPublic, Visibility, parse_macro_input, punctuated::Punctuated, token::{Brace, Bracket, Colon, Comma, Enum, FatArrow, Gt, Lt, Match, Paren, Pound, Pub}};

use once_cell::sync::Lazy;

static MAP: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[proc_macro_attribute]
pub fn generate_exchanges(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_mod = parse_macro_input!(input as ItemMod);
    let default = parse_macro_input!(args as Path);

    let mut v = Punctuated::new();
    if let Some((_, contents)) = &item_mod.content {
        for content in contents {
            if let Item::Const(item) = content {
                let i = &item.ident;
                v.push(Expr::Verbatim(quote! { #i }));
            }
        }
    }

    let array = ExprArray {
        attrs: Vec::new(),
        bracket_token: Bracket::default(),
        elems: v
    };
    let name = &item_mod.ident;
    {
        let mut lock = MAP.write().unwrap();
        lock.push(name.to_string());
    }
    return quote! {
        pub mod #name {
            #item_mod

            pub use #name::*;

            use once_cell::sync::Lazy;

            use rusty_money::{Exchange, ExchangeRate, Money, MoneyError, FormattableCurrency};

            use rust_decimal::{Decimal, MathematicalOps};

            static RATES: Lazy<Vec<ExchangeRate<Currency>>> = Lazy::new(|| {
                let mut v = Vec::new();
                let currencies = #array;
                for c1 in &currencies {
                    for c2 in &currencies {
                        if c1 != c2 {
                            v.push(ExchangeRate::new(*c1, *c2, Decimal::new(10, 0).powi((c1.exponent as i64) - (c2.exponent as i64))).unwrap());
                        }
                    }
                }
                v
            });

            static EXCHANGE: Lazy<Exchange<'static, Currency>> = Lazy::new(|| {
                let mut exchange = Exchange::new();
                for rate in RATES.iter() {
                    exchange.set_rate(&rate);
                }
                exchange
            });

            pub fn convert<'a>(from: Money<'a, Currency>, to: &Currency) -> Result<Money<'a, Currency>, MoneyError> {
                if let Some(rate) = EXCHANGE.get_rate(from.currency(), to) {
                    rate.convert(from)
                }
                else {
                    Err(MoneyError::InvalidCurrency)
                }
            }

            pub fn load<'a>(value: Decimal) -> Money<'a, Currency> {
                Money::from_decimal(value, #default)
            }

            pub fn save<'a>(value: Money<'a, Currency>) -> Result<Decimal, MoneyError> {
                let default_currency = if value.currency() != #default {
                    convert(value, #default)?
                }
                else {
                    value
                };
                Ok(*default_currency.amount())
            }
        }
    }
    .into();
}

#[proc_macro]
pub fn generate_map(_: TokenStream) -> TokenStream {
    // create a GenericMoney enum
    let mut gm_enum = ItemEnum {
        attrs: vec![Attribute {
            pound_token: Pound::default(),
            style: AttrStyle::Outer,
            bracket_token: Bracket::default(),
            path: Path {
                leading_colon: None,
                segments: {
                    let mut s = Punctuated::new();
                    s.push(PathSegment {
                        ident: Ident::new("derive", Span::call_site()),
                        arguments: PathArguments::None,
                    });
                    s
                }
            },
            tokens: quote! { (Debug, PartialEq) },
        }],
        vis: Visibility::Public(VisPublic {
            pub_token: Pub::default(),
        }),
        enum_token: Enum::default(),
        ident: Ident::new("GenericMoney", Span::call_site()),
        generics: Generics {
            lt_token: Some(Lt::default()),
            params: {
                let mut p = Punctuated::new();
                p.push(GenericParam::Lifetime(LifetimeDef {
                    attrs: Vec::new(),
                    lifetime: Lifetime {
                        apostrophe: Span::call_site(),
                        ident: Ident::new("a", Span::call_site()),
                    },
                    colon_token: None,
                    bounds: Punctuated::new(),
                }));
                p
            },
            gt_token: Some(Gt::default()),
            where_clause: None,
        },
        brace_token: Brace::default(),
        variants: Punctuated::new(),
    };
    // create a GenericCurrency enum
    let mut gc_enum = ItemEnum {
        attrs: vec![Attribute {
            pound_token: Pound::default(),
            style: AttrStyle::Outer,
            bracket_token: Bracket::default(),
            path: Path {
                leading_colon: None,
                segments: {
                    let mut s = Punctuated::new();
                    s.push(PathSegment {
                        ident: Ident::new("derive", Span::call_site()),
                        arguments: PathArguments::None,
                    });
                    s
                }
            },
            tokens: quote! { (Debug, PartialEq) },
        }],
        vis: Visibility::Public(VisPublic {
            pub_token: Pub::default(),
        }),
        enum_token: Enum::default(),
        ident: Ident::new("GenericCurrency", Span::call_site()),
        generics: Generics {
            lt_token: Some(Lt::default()),
            params: {
                let mut p = Punctuated::new();
                p.push(GenericParam::Lifetime(LifetimeDef {
                    attrs: Vec::new(),
                    lifetime: Lifetime {
                        apostrophe: Span::call_site(),
                        ident: Ident::new("a", Span::call_site()),
                    },
                    colon_token: None,
                    bounds: Punctuated::new(),
                }));
                p
            },
            gt_token: Some(Gt::default()),
            where_clause: None,
        },
        brace_token: Brace::default(),
        variants: Punctuated::new(),
    };
    // create a match expression
    let mut load_match = ExprMatch {
        attrs: Vec::new(),
        match_token: Match::default(),
        expr: Box::new(Expr::Verbatim(quote!{ family })),
        brace_token: Brace::default(),
        arms: Vec::new(),
    };
    // create a match expression
    let mut save_match = ExprMatch {
        attrs: Vec::new(),
        match_token: Match::default(),
        expr: Box::new(Expr::Verbatim(quote!{ value })),
        brace_token: Brace::default(),
        arms: Vec::new(),
    };
    // create a match expression
    let mut convert_match = ExprMatch {
        attrs: Vec::new(),
        match_token: Match::default(),
        expr: Box::new(Expr::Verbatim(quote!{ (from, to) })),
        brace_token: Brace::default(),
        arms: Vec::new(),
    };
    // add an arm for every module
    for m in MAP.read().unwrap().iter() {
        let i = Ident::new(m, Span::call_site());
        gm_enum.variants.push(Variant {
            attrs: Vec::new(),
            ident: i.clone(),
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: Paren::default(),
                unnamed: {
                    let mut u = Punctuated::new();
                    u.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        ident: None,
                        colon_token: Some(Colon::default()),
                        ty: Type::Verbatim(quote! { Money<'a, #i::Currency> }),
                    });
                    u
                },
            }),
            discriminant: None,
        });
        gc_enum.variants.push(Variant {
            attrs: Vec::new(),
            ident: i.clone(),
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: Paren::default(),
                unnamed: {
                    let mut u = Punctuated::new();
                    u.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        ident: None,
                        colon_token: Some(Colon::default()),
                        ty: Type::Verbatim(quote! { &'a #i::Currency }),
                    });
                    u
                },
            }),
            discriminant: None,
        });
        load_match.arms.push(Arm {
            attrs: Vec::new(),
            pat: Pat::Lit(PatLit {
                attrs: Vec::new(),
                expr: Box::new(Expr::Verbatim(quote!{ #m })),
            }),
            guard: None,
            fat_arrow_token: FatArrow::default(),
            body: Box::new(Expr::Verbatim(quote!{ Some(GenericMoney::#i(#i::load(qty))) })),
            comma: Some(Comma::default()),
        });
        save_match.arms.push(Arm {
            attrs: Vec::new(),
            pat: Pat::Lit(PatLit {
                attrs: Vec::new(),
                expr: Box::new(Expr::Verbatim(quote!{ GenericMoney::#i(m) })),
            }),
            guard: None,
            fat_arrow_token: FatArrow::default(),
            body: Box::new(Expr::Verbatim(quote!{ #i::save(m) })),
            comma: Some(Comma::default()),
        });
        convert_match.arms.push(Arm {
            attrs: Vec::new(),
            pat: Pat::Lit(PatLit {
                attrs: Vec::new(),
                expr: Box::new(Expr::Verbatim(quote!{ (GenericMoney::#i(m), GenericCurrency::#i(c)) })),
            }),
            guard: None,
            fat_arrow_token: FatArrow::default(),
            body: Box::new(Expr::Verbatim(quote!{ Ok(GenericMoney::#i(#i::convert(m, c)?)) })),
            comma: Some(Comma::default()),
        });
    }
    // create a fallback arm
    load_match.arms.push(Arm {
        attrs: Vec::new(),
        pat: Pat::Lit(PatLit {
            attrs: Vec::new(),
            expr: Box::new(Expr::Verbatim(quote!{ _ })),
        }),
        guard: None,
        fat_arrow_token: FatArrow::default(),
        body: Box::new(Expr::Verbatim(quote!{ None })),
        comma: Some(Comma::default()),
    });
    // create a fallback arm
    convert_match.arms.push(Arm {
        attrs: Vec::new(),
        pat: Pat::Lit(PatLit {
            attrs: Vec::new(),
            expr: Box::new(Expr::Verbatim(quote!{ _ })),
        }),
        guard: None,
        fat_arrow_token: FatArrow::default(),
        body: Box::new(Expr::Verbatim(quote!{ Err(MoneyError::InvalidCurrency) })),
        comma: Some(Comma::default()),
    });
    // output result
    return quote! {
        use rusty_money::MoneyError;

        #gc_enum

        #gm_enum

        pub fn load<'a>(family: &str, qty: Decimal) -> Option<GenericMoney<'a>> {
            #load_match
        }

        pub fn save<'a>(value: GenericMoney<'a>) -> Result<Decimal, MoneyError> {
            #save_match
        }

        pub fn convert<'a>(from: GenericMoney<'a>, to: GenericCurrency<'a>) -> Result<GenericMoney<'a>, MoneyError> {
            #convert_match
        }
    }.into();
}
