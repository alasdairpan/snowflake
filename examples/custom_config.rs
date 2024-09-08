use {std::error::Error, twitter_snowflake::Snowflake};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let worker_id_bits = 4;
    let epoch: u64 = 1609459200000; // 2021-01-01 00:00:00.000 UTC

    let mut snowflake = Snowflake::builder()
        .with_worker_id_bits(worker_id_bits)
        .with_worker_id(worker_id)
        .with_epoch(epoch)
        .build()?;

    let sfid = snowflake.generate()?;
    println!("Snowflake ID: {}", sfid);
    Ok(())
}
