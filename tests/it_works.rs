
use generate_exchanges::{generate_exchanges, generate_map};

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
    },
    #[generate_exchanges(GWEI)]
    ether {
        ETH: {
            code: "ETH",
            exponent: 18,
            locale: Locale::EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "ETH",
            symbol: "ETH",
            symbol_first: false,
        },
        FINNEY: {
            code: "finney",
            exponent: 15,
            locale: Locale::EnUs,
            minor_units: 1_000_000_000_000_000,
            name: "finney",
            symbol: "finney",
            symbol_first: false,
        },
        SZABO: {
            code: "szabo",
            exponent: 12,
            locale: Locale::EnUs,
            minor_units: 1_000_000_000_000,
            name: "szabo",
            symbol: "szabo",
            symbol_first: false,
        },
        GWEI: {
            code: "gwei",
            exponent: 9,
            locale: Locale::EnUs,
            minor_units: 1_000_000_000,
            name: "gwei",
            symbol: "gwei",
            symbol_first: false,
        },
        WEI: {
            code: "wei",
            exponent: 0,
            locale: Locale::EnUs,
            minor_units: 1,
            name: "wei",
            symbol: "wei",
            symbol_first: false,
        }
    }
);

generate_map!();

#[test]
fn direct() {
    let value = Decimal::new(1, 0);
    let value1 = btc::load(value);
    assert_eq!(value1, Money::from_major(1, btc::BTC));
    let value2 = btc::convert(value1, btc::SATOSHI).unwrap();
    assert_eq!(value2, Money::from_major(100000000, btc::SATOSHI));
    assert_eq!(value, btc::save(value2).unwrap());
}

#[test]
fn indirect() {
    let value = Decimal::new(1, 0);
    let value1 = load("btc", value).unwrap();
    assert_eq!(value1, GenericMoney::btc(Money::from_major(1, btc::BTC)));
    let value2 = convert(value1, GenericCurrency::btc(btc::SATOSHI)).unwrap();
    assert_eq!(value2, GenericMoney::btc(Money::from_major(100000000, btc::SATOSHI)));
    assert_eq!(value, save(value2).unwrap());
}
