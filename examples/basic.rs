use {std::error::Error, twitter_snowflake::Snowflake};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id)?;
    let sfid = snowflake.generate()?;
    println!("Snowflake ID: {}", sfid);

    Ok(())
}
