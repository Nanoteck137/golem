use std::{time::Duration, sync::RwLock};

use rocket::tokio::{spawn, time};

static NUMBER: RwLock<u32> = RwLock::new(0);

#[macro_use]
extern crate rocket;

#[get("/")]
fn test() -> String {
    let number = NUMBER.read().unwrap();
    format!("Number: {}", number)
}

#[launch]
fn rocket() -> _ {
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            {
                let mut lock = NUMBER.write().unwrap();
                *lock = *lock + 1;
            }
        }
    });

    rocket::build().mount("/test", routes![test])
}
