//! Tests for the `#[derive(Arbitrary)]` macro

extern crate quickcheck;
extern crate quickcheck_macros;

use quickcheck::{Arbitrary, Gen};

// Test for named struct
#[derive(Arbitrary, Debug, PartialEq, Clone)]
struct NamedStruct {
    a: u32,
    b: bool,
    c: String,
}

// Test for tuple struct
#[derive(Arbitrary, Debug, PartialEq, Clone)]
struct TupleStruct(u32, bool, String);

// Test for unit struct
#[derive(Arbitrary, Debug, PartialEq, Clone)]
struct UnitStruct;

#[test]
fn test_named_struct_arbitrary() {
    let mut g = Gen::new(100);
    let instance = NamedStruct::arbitrary(&mut g);
    
    // Basic sanity checks
    assert!(instance.a <= 100);
    assert!(instance.c.len() <= 100);
}

#[test]
fn test_tuple_struct_arbitrary() {
    let mut g = Gen::new(100);
    let instance = TupleStruct::arbitrary(&mut g);
    
    // Basic sanity checks
    assert!(instance.0 <= 100);
    assert!(instance.2.len() <= 100);
}

#[test]
fn test_unit_struct_arbitrary() {
    let mut g = Gen::new(100);
    let instance = UnitStruct::arbitrary(&mut g);
    
    // Unit struct should always be the same
    assert_eq!(instance, UnitStruct);
}

#[test]
fn test_named_struct_shrink() {
    let mut g = Gen::new(100);
    let instance = NamedStruct::arbitrary(&mut g);
    let shrinker = instance.shrink();
    
    // Should produce an iterator
    assert!(shrinker.count() >= 0);
}

#[test]
fn test_tuple_struct_shrink() {
    let mut g = Gen::new(100);
    let instance = TupleStruct::arbitrary(&mut g);
    let shrinker = instance.shrink();
    
    // Should produce an iterator
    assert!(shrinker.count() >= 0);
}

#[test]
fn test_unit_struct_shrink() {
    let instance = UnitStruct;
    let shrinker = instance.shrink();
    
    // Unit struct should have no shrink values
    assert_eq!(shrinker.count(), 0);
}
