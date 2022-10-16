use std::path::Path;
use std::time::Instant;

use esp_remotecontrolled_led_matrix_client::sync::Client;

fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");

    let image_path = std::env::var("IMAGE");
    let image_path = image_path.as_deref().unwrap_or("bla.png");

    let mut client = Client::connect(addr).expect("connection error");

    println!(
        "{} size {}x{} = {} pixels",
        addr,
        client.width(),
        client.height(),
        client.total_pixels()
    );

    let load = Instant::now();

    let img = load_img(image_path, client.width(), client.height()).unwrap();

    let x = (client.width() - img.0) / 2;
    let y = (client.height() - img.1) / 2;

    println!("img1 {img:?}");

    let load = load.elapsed();
    println!("Bitmap load took {:9.2} ms", load.as_secs_f64() * 1000.0);

    let write = Instant::now();
    client.contiguous(x, y, img.0, img.1, &img.2).unwrap();
    client.flush().unwrap();
    let write = write.elapsed();
    println!("Bitmap send took {:9.2} ms", write.as_secs_f64() * 1000.0);
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
