extern crate rsedis;

use std;

use rsedis::database::Database;
use rsedis::database::Value;
use rsedis::util::mstime;

#[test]
fn set_get() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = vec![1u8, 2, 3, 4];
    let expected = Vec::clone(&value);
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert_eq!(database.get(0, &key).unwrap(), &Value::Data(expected));
}

#[test]
fn get_empty() {
    let database = Database::new();
    let key = vec![1u8];
    assert!(database.get(0, &key).is_none());
}

#[test]
fn set_set_get() {
    let mut database = Database::new();
    let key = vec![1u8];
    assert!(database.get_or_create(0, &key).set(vec![0u8, 0, 0]).is_ok());
    let value = vec![1u8, 2, 3, 4];
    let expected = Vec::clone(&value);
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert_eq!(database.get(0, &key).unwrap(), &Value::Data(expected));
}

#[test]
fn append_append_get() {
    let mut database = Database::new();
    let key = vec![1u8];
    assert_eq!(database.get_or_create(0, &key).append(vec![0u8, 0, 0]).unwrap(), 3);
    assert_eq!(database.get_or_create(0, &key).append(vec![1u8, 2, 3, 4]).unwrap(), 7);
    assert_eq!(database.get(0, &key).unwrap(), &Value::Data(vec![0u8, 0, 0, 1, 2, 3, 4]));
}

#[test]
fn set_number() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = b"123".to_vec();
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert_eq!(database.get(0, &key).unwrap(), &Value::Integer(123));
}

#[test]
fn append_number() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = b"123".to_vec();
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert_eq!(database.get_or_create(0, &key).append(b"asd".to_vec()).unwrap(), 6);
    assert_eq!(database.get(0, &key).unwrap(), &Value::Data(b"123asd".to_vec()));
}

#[test]
fn remove_value() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = vec![1u8, 2, 3, 4];
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    database.remove(0, &key).unwrap();
    assert!(database.remove(0, &key).is_none());
}

#[test]
fn incr_str() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = b"123".to_vec();
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert_eq!(database.get_or_create(0, &key).incr(1).unwrap(), 124);
    assert_eq!(database.get(0, &key).unwrap(), &Value::Integer(124));
}

#[test]
fn incr_create_update() {
    let mut database = Database::new();
    let key = vec![1u8];
    assert_eq!(database.get_or_create(0, &key).incr(124).unwrap(), 124);
    assert_eq!(database.get(0, &key).unwrap(), &Value::Integer(124));
    assert_eq!(database.get_or_create(0, &key).incr(1).unwrap(), 125);
    assert_eq!(database.get(0, &key).unwrap(), &Value::Integer(125));
}

#[test]
fn incr_overflow() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = format!("{}", std::i64::MAX).into_bytes();
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    assert!(database.get_or_create(0, &key).incr(1).is_err());
}

#[test]
fn set_expire_get() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = vec![1u8, 2, 3, 4];
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    database.set_msexpiration(0, key.clone(), mstime());
    assert_eq!(database.get(0, &key), None);
}

#[test]
fn set_will_expire_get() {
    let mut database = Database::new();
    let key = vec![1u8];
    let value = vec![1u8, 2, 3, 4];
    let expected = Vec::clone(&value);
    assert!(database.get_or_create(0, &key).set(value).is_ok());
    database.set_msexpiration(0, key.clone(), mstime() + 10000);
    assert_eq!(database.get(0, &key), Some(&Value::Data(expected)));
}

#[test]
fn getrange_integer() {
    let value = Value::Integer(123);
    assert_eq!(value.getrange(0, -1).unwrap(), "123".to_owned().into_bytes());
    assert_eq!(value.getrange(-100, -2).unwrap(), "12".to_owned().into_bytes());
    assert_eq!(value.getrange(1, 1).unwrap(), "2".to_owned().into_bytes());
}

#[test]
fn getrange_data() {
    let value = Value::Data(vec![1,2,3]);
    assert_eq!(value.getrange(0, -1).unwrap(), vec![1,2,3]);
    assert_eq!(value.getrange(-100, -2).unwrap(), vec![1,2]);
    assert_eq!(value.getrange(1, 1).unwrap(), vec![2]);
}

#[test]
fn setrange_append() {
    let mut value = Value::Data(vec![1,2,3]);
    assert_eq!(value.setrange(3, vec![4, 5, 6]).unwrap(), 6);
    assert_eq!(value, Value::Data(vec![1,2,3,4,5,6]));
}

#[test]
fn setrange_create() {
    let mut value = Value::Nil;
    assert_eq!(value.setrange(0, vec![4, 5, 6]).unwrap(), 3);
    assert_eq!(value, Value::Data(vec![4,5,6]));
}

#[test]
fn setrange_padding() {
    let mut value = Value::Data(vec![1,2,3]);
    assert_eq!(value.setrange(5, vec![6]).unwrap(), 6);
    assert_eq!(value, Value::Data(vec![1,2,3,0,0,6]));
}

#[test]
fn setrange_intermediate() {
    let mut value = Value::Data(vec![1,2,3,4,5]);
    assert_eq!(value.setrange(2, vec![13, 14]).unwrap(), 5);
    assert_eq!(value, Value::Data(vec![1,2,13,14,5]));
}
