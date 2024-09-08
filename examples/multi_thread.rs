use {
    snowflake::Snowflake,
    std::{
        error::Error,
        sync::{mpsc, Arc, Mutex},
        thread,
        time::Duration,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let snowflake = Arc::new(Mutex::new(Snowflake::new(worker_id)?));
    let (tx, rx) = mpsc::channel();

    for _ in 0 .. 10 {
        let snowflake = Arc::clone(&snowflake);
        let tx = tx.clone();

        thread::spawn(move || {
            if let Ok(mut guard) = snowflake.lock() {
                match guard.generate() {
                    Ok(sfid) => {
                        let _ = tx.send(sfid);
                    }
                    Err(e) => {
                        println!("Generate error: {}", e);
                    }
                }
            }
        });
    }

    thread::sleep(Duration::from_secs(1));

    for _ in 0 .. 10 {
        let sfid = rx.recv()?;
        println!("Snowflake ID: {}", sfid);
    }

    Ok(())
}
