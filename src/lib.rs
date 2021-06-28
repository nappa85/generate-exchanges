
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprPath, PathArguments, parse_macro_input, punctuated::Punctuated, ItemMod, Item, PathSegment, Path, token::Bracket};

#[proc_macro_attribute]
pub fn generate_exchanges(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_mod = parse_macro_input!(input as ItemMod);
    let default = parse_macro_input!(args as Path);

    let mut v = Punctuated::new();
    if let Some((_, contents)) = &item_mod.content {
        for content in contents {
            if let Item::Const(item) = content {
                let mut segments = Punctuated::new();
                segments.push(PathSegment {
                    ident: item.ident.clone(),
                    arguments: PathArguments::None,
                });
                v.push(Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments
                    }
                }));
            }
        }
    }

    let array = ExprArray {
        attrs: Vec::new(),
        bracket_token: Bracket::default(),
        elems: v
    };
    let name = &item_mod.ident;
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
