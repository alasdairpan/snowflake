use snowflake::Snowflake;

fn main() {
    let node_id = 1;
    let mut snowflake = Snowflake::new(node_id).unwrap();

    let id1 = snowflake.generate().unwrap();
    let id2 = snowflake.generate().unwrap();
    let id3 = snowflake.generate().unwrap();

    println!("id1: {}", id1);
    println!("id2: {}", id2);
    println!("id3: {}", id3);

    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);
}

// Output:
// id1: 46186916520919041
// id2: 46186916520919042
// id3: 46186916520919043
