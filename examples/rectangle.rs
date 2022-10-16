use std::time::Instant;

use esp_remotecontrolled_led_matrix_client::sync::Client;

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

    let write = Instant::now();

    // Rectangle at 3x1 with 3x4 size will be filled with RGB 0 0 255 = blue
    // Basically should look like this:
    //  --------
    // |
    // |   XXX
    // |   XXX
    // |   XXX
    // |   XXX
    // |

    client.rectangle(3, 1, 3, 4, 0, 0, 255).unwrap();
    client.flush().unwrap();

    let took = write.elapsed();
    println!("Rectangle took {:9.2} ms", took.as_secs_f64() * 1000.0);
}
