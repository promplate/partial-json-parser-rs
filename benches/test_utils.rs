use prop::strategy::ValueTree;
use proptest::prelude::*;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Json {
    Null,
    NaN,
    Infinity,
    NInfinity,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Map(HashMap<String, Json>),
}

impl Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Json::Null => serializer.serialize_unit(),
            Json::NaN => serializer.serialize_f64(f64::NAN),
            Json::Infinity => serializer.serialize_f64(f64::INFINITY),
            Json::NInfinity => serializer.serialize_f64(f64::NEG_INFINITY),
            Json::Bool(b) => serializer.serialize_bool(*b),
            Json::Number(n) => serializer.serialize_f64(*n),
            Json::String(s) => serializer.serialize_str(s),
            Json::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for element in arr {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            Json::Map(map) => {
                let mut map_serializer = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    map_serializer.serialize_entry(k, v)?;
                }
                map_serializer.end()
            }
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = json5::to_string(self).unwrap();
        write!(f, "{}", &s)
    }
}

pub fn arb_json() -> impl Strategy<Value = Json> {
    let leaf = prop_oneof![
        Just(Json::Null),
        Just(Json::NaN),
        Just(Json::Infinity),
        Just(Json::NInfinity),
        any::<bool>().prop_map(Json::Bool),
        any::<f64>()
            .prop_filter("Must not be an integer", |&x| x.fract() != 0.0)
            .prop_map(Json::Number),
        ".*".prop_map(Json::String),
    ];
    leaf.prop_recursive(
        8,   // 8 levels deep
        256, // Shoot for maximum size of 256 nodes
        10,  // We put up to 10 items per collection
        |inner| {
            prop_oneof![
                // Take the inner strategy and make the two recursive cases.
                prop::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
                prop::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map),
            ]
        },
    )
}

pub fn truncate_middle(input: &str) -> String {
    // 计算字符长度
    let len = input.chars().count();
    if len < 2 {
        return input.to_string();
    }

    // 计算保留前后各一半字符的索引
    let half_len = len / 2;

    // 找到前半部分的结束字节索引
    let mut mid_byte_index = 0;
    for (i, (byte_index, _)) in input.char_indices().enumerate() {
        if i == half_len {
            mid_byte_index = byte_index;
            break;
        }
    }

    // 构造新的字符串
    input[..mid_byte_index].to_string()
}

fn is_valid_json(json_str: &str) -> bool {
    !matches!(json5::from_str::<Value>(json_str), Err(_err))
}

pub fn gen_test_cases(num: usize) -> Vec<String> {
    // 生成一些测试用例
    let mut test_cases = vec![];
    for _ in 0..num {
        let mut case;
        loop {
            case = arb_json()
                .new_tree(&mut proptest::test_runner::TestRunner::default())
                .unwrap()
                .current()
                .to_string();
            if is_valid_json(&case) {
                break;
            }
        }
        let case = truncate_middle(&case);
        test_cases.push(case);
    }

    test_cases
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_play() {
        let json_null = Json::Null;
        let json_bool = Json::Bool(true);
        let json_number = Json::Number(42.0);
        let json_string = Json::String("Hello, world!".to_string());
        let json_array = Json::Array(vec![Json::Number(1.0), Json::Bool(false)]);
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Json::String("value1".to_string()));
        map.insert("key2".to_string(), Json::Number(10.0));
        // let json_map = Json::Map(map);

        // println!("{}", json_null);
        // println!("{}", json_bool);
        // println!("{}", json_number);
        // println!("{}", json_string);
        // println!("{}", json_array);
        // println!("{}", json_map);

        assert!(json_null.to_string() == "null");
        assert!(json_bool.to_string() == "true");
        assert!(json_number.to_string() == "42");
        assert!(json_string.to_string() == r#""Hello, world!""#);
        assert!(json_array.to_string() == "[1,false]");
        // 这个由于map是无序的，所以确实测不了
        // assert!(json_map.to_string() == r#"{"key1":"value1","key2":10.0}"#);
    }
}
