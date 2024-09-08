use {snowflake::Snowflake, std::error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let worker_id_bits = Some(4);
    let epoch: Option<u64> = Some(1609459200000); // 2021-01-01 00:00:00.000 UTC
    let mut snowflake = Snowflake::with_config(worker_id, worker_id_bits, None, epoch)?;
    let sfid = snowflake.generate()?;
    println!("Snowflake ID: {}", sfid);
    Ok(())
}
