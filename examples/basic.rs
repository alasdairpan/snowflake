use twitter_snowflake::Snowflake;

fn main() {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    let sfid = snowflake.generate().unwrap();
    println!("Snowflake ID: {}", sfid);
}
