use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use esp_wlan_led_matrix_client::sync::Client;

fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");
    let mut client = Client::connect(addr).expect("connection error");

    println!(
        "{} size {}x{} = {} pixels",
        addr,
        client.width(),
        client.height(),
        client.total_pixels()
    );

    let load = Instant::now();

    let img1 = load_img("bla-1.png", client.width(), client.height()).unwrap();
    let img2 = load_img("bla-2.png", client.width(), client.height()).unwrap();

    let load = load.elapsed();
    println!("Bitmap load took {:9.2} ms", load.as_secs_f64() * 1000.0);

    loop {
        const SLEEP_TIME: Duration = Duration::from_millis(1000 / 5);

        let write = Instant::now();
        client.contiguous(0, 0, img1.0, img1.1, &img1.2).unwrap();
        client.flush().unwrap();
        let write = write.elapsed();
        println!("Bitmap send took {:9.2} ms", write.as_secs_f64() * 1000.0);

        sleep(SLEEP_TIME);

        let write = Instant::now();
        client.contiguous(0, 0, img2.0, img2.1, &img2.2).unwrap();
        client.flush().unwrap();
        let write = write.elapsed();
        println!("Bitmap send took {:9.2} ms", write.as_secs_f64() * 1000.0);

        sleep(SLEEP_TIME);
    }
}

#[allow(clippy::cast_possible_truncation)]
fn load_img<P>(path: P, max_width: u8, max_height: u8) -> anyhow::Result<(u8, u8, Vec<u8>)>
where
    P: AsRef<Path>,
{
    let img = image::io::Reader::open(path)?.decode()?;

    let width = img.width().min(u32::from(max_width)) as u8;
    let height = img.height().min(u32::from(max_height)) as u8;

    let buffer = img
        .to_rgb8()
        .enumerate_pixels()
        .filter(|(x, y, _color)| x < &u32::from(max_width) && y < &u32::from(max_height))
        .flat_map(|(_x, _y, color)| [color.0[0], color.0[1], color.0[2]])
        .collect::<Vec<_>>();
    Ok((width, height, buffer))
}
