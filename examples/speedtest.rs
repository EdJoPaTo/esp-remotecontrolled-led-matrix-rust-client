use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use esp_remotecontrolled_led_matrix_client::sync::Client;

fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");

    loop {
        match Client::connect(addr) {
            Ok(mut client) => {
                println!(
                    "{} size {}x{} = {} pixels",
                    addr,
                    client.width(),
                    client.height(),
                    client.total_pixels()
                );

                if let Err(err) = speedtest(&mut client) {
                    eprintln!("ERROR: {err}");
                }
            }
            Err(err) => {
                eprintln!("CONNECT ERROR: {err}");
                sleep(Duration::from_millis(500));
            }
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn speedtest(client: &mut Client) -> std::io::Result<()> {
    let total_pixels = client.total_pixels() as usize;
    let start = Instant::now();
    let mut pixel_wrote: usize = 0;

    loop {
        let write = Instant::now();

        for y in 0..client.height() {
            for x in 0..client.width() {
                let red = rand::random::<u8>();
                let green = rand::random::<u8>() / 2;
                let blue = rand::random::<u8>() / 3;

                client.pixel(x, y, red, green, blue)?;
                pixel_wrote = pixel_wrote.overflowing_add(1).0;
            }
        }

        client.flush()?;

        let took = write.elapsed();

        let pixel_per_second = (pixel_wrote as f64) / start.elapsed().as_secs_f64();
        let screens_per_second =
            (pixel_wrote / total_pixels) as f64 / start.elapsed().as_secs_f64();
        let kb_per_second = pixel_per_second * 6.0 / 1024.0;

        println!(
            "{:6.1}s since start; took {:9.2} ms for a screen; Average:{:12.1} pixels / second {:9.3} screens / second {:9.3} kB/s",
            start.elapsed().as_secs_f64(),
            took.as_secs_f64() * 1000.0,
            pixel_per_second,
            screens_per_second,
            kb_per_second,
        );
    }
}
