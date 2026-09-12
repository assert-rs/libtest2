#![cfg(feature = "alloc")]
#![allow(clippy::dbg_macro)] // unsure why config isn't working

use snapbox::prelude::*;
use snapbox::str;

use json_write::ToJsonKey;
use json_write::ToJsonValue;

#[track_caller]
fn t<N: ToJsonValue + core::fmt::Debug>(value: N, expected: impl IntoData) {
    let key = "key".to_json_key();
    let string = value.to_json_value();
    let object = format!("{{ {key}: {string} }}");
    let parsed = format!("{:?}", object.parse::<serde_json::Value>());
    let results = Results {
        value,
        string,
        parsed,
    };
    snapbox::assert_data_eq!(results.to_debug(), expected.raw());
}

#[derive(Debug)]
#[allow(dead_code)]
struct Results<N: core::fmt::Debug> {
    value: N,
    string: String,
    parsed: String,
}

#[test]
fn zero() {
    t(
        0.0_f64,
        str![[r#"
Results {
    value: 0.0,
    string: "0.0",
    parsed: "Ok(Object {\"key\": Number(0.0)})",
}

"#]],
    );
}

#[test]
fn neg_zero() {
    t(
        -0.0_f64,
        str![[r#"
Results {
    value: -0.0,
    string: "-0.0",
    parsed: "Ok(Object {\"key\": Number(-0.0)})",
}

"#]],
    );
}

#[test]
fn inf() {
    t(
        f64::INFINITY,
        str![[r#"
Results {
    value: inf,
    string: "null",
    parsed: "Ok(Object {\"key\": Null})",
}

"#]],
    );
}

#[test]
fn neg_inf() {
    t(
        f64::NEG_INFINITY,
        str![[r#"
Results {
    value: -inf,
    string: "null",
    parsed: "Ok(Object {\"key\": Null})",
}

"#]],
    );
}

#[test]
fn nan() {
    t(
        f64::NAN.copysign(1.0),
        str![[r#"
Results {
    value: NaN,
    string: "null",
    parsed: "Ok(Object {\"key\": Null})",
}

"#]],
    );
}

#[test]
fn neg_nan() {
    t(
        f64::NAN.copysign(-1.0),
        str![[r#"
Results {
    value: NaN,
    string: "null",
    parsed: "Ok(Object {\"key\": Null})",
}

"#]],
    );
}
