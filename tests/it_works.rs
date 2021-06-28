
use generate_exchanges::generate_exchanges;

use rust_decimal::Decimal;

use rusty_money::{Money, define_currency_set};

define_currency_set!(
    #[generate_exchanges(BTC)]
    btc {
        BTC: {
            code: "BTC",
            exponent: 8,
            locale: Locale::EnUs,
            minor_units: 100_000_000,
            name: "BTC",
            symbol: "BTC",
            symbol_first: true,
        },
        MBTC: {
            code: "mBTC",
            exponent: 5,
            locale: Locale::EnUs,
            minor_units: 100_000,
            name: "mBTC",
            symbol: "mBTC",
            symbol_first: true,
        },
        UBTC: {
            code: "uBTC",
            exponent: 2,
            locale: Locale::EnUs,
            minor_units: 100,
            name: "uBTC",
            symbol: "uBTC",
            symbol_first: true,
        },
        SATOSHI: {
            code: "satoshi",
            exponent: 0,
            locale: Locale::EnUs,
            minor_units: 1,
            name: "satoshi",
            symbol: "satoshi",
            symbol_first: true,
        }
    }
);

#[test]
fn it_works() {
    let value = Decimal::new(1, 0);
    let value1 = btc::load(value);
    assert_eq!(value1, Money::from_major(1, btc::BTC));
    let value2 = btc::convert(value1, btc::SATOSHI).unwrap();
    assert_eq!(value2, Money::from_major(100000000, btc::SATOSHI));
    assert_eq!(value, btc::save(value2).unwrap());
}
