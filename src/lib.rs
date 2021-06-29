use std::sync::RwLock;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Expr, ExprArray, Item, ItemMod, Path, parse_macro_input, punctuated::Punctuated, token::Bracket};

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
    let lock = MAP.read().unwrap();
    // create GenericCurrency enum body
    let gc_enum = lock.iter().map(|m| {
        let i = Ident::new(m, Span::call_site());
        quote! { #i(&'a #i::Currency), }
    }).collect::<TokenStream2>();
    // create GenericMoney enum body
    let gm_enum = lock.iter().map(|m| {
        let i = Ident::new(m, Span::call_site());
        quote! { #i(Money<'a, #i::Currency>), }
    }).collect::<TokenStream2>();
    // create load method match body
    let load_match = lock.iter().map(|m| {
        let i = Ident::new(m, Span::call_site());
        quote! { #m => Some(GenericMoney::#i(#i::load(qty))), }
    }).collect::<TokenStream2>();
    // create save method match body
    let save_match = lock.iter().map(|m| {
        let i = Ident::new(m, Span::call_site());
        quote! { GenericMoney::#i(m) => #i::save(m), }
    }).collect::<TokenStream2>();
    // create convert method match body
    let convert_match = lock.iter().map(|m| {
        let i = Ident::new(m, Span::call_site());
        quote! { (GenericMoney::#i(m), GenericCurrency::#i(c)) => Ok(GenericMoney::#i(#i::convert(m, c)?)), }
    }).collect::<TokenStream2>();
    // output result
    return quote! {
        use rusty_money::MoneyError;

        #[derive(Debug, PartialEq)]
        pub enum GenericCurrency<'a> {
            #gc_enum
        }

        #[derive(Debug, PartialEq)]
        pub enum GenericMoney<'a> {
            #gm_enum
        }

        pub fn load<'a>(family: &str, qty: Decimal) -> Option<GenericMoney<'a>> {
            match family {
                #load_match
                _ => None
            }
        }

        pub fn save<'a>(value: GenericMoney<'a>) -> Result<Decimal, MoneyError> {
            match value {
                #save_match
            }
        }

        pub fn convert<'a>(from: GenericMoney<'a>, to: GenericCurrency<'a>) -> Result<GenericMoney<'a>, MoneyError> {
            match (from, to) {
                #convert_match
                _ => Err(MoneyError::InvalidCurrency),
            }
        }
    }.into();
}
