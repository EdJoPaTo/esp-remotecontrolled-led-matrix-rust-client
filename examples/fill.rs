use std::time::Instant;

use esp_wlan_led_matrix_client::sync::Client;

fn main() {
    let addr = "espPixelmatrix:1337";
    let mut client = Client::connect(addr).expect("connection error");

    println!(
        "size {}x{} = {} pixels",
        client.width(),
        client.height(),
        client.total_pixels()
    );

    let write = Instant::now();

    // Fill with RGB 0 0 255 = blue
    client.fill(0, 0, 255).unwrap();
    client.flush().unwrap();

    let took = write.elapsed();
    println!("Fill took {:9.2} ms", took.as_secs_f64() * 1000.0);
}
