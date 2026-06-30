use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use twitter_snowflake::Snowflake;

fn main() {
    let worker_id = 1;
    let snowflake = Arc::new(Mutex::new(Snowflake::new(worker_id).unwrap()));
    let (tx, rx) = mpsc::channel();

    for _ in 0..10 {
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

    for _ in 0..10 {
        let sfid = rx.recv().unwrap();
        println!("Snowflake ID: {}", sfid);
    }

    // All 10 IDs have been received — join is not strictly needed here
    // since the threads will terminate when the process exits, but
    // real applications should wait on handles.
}
