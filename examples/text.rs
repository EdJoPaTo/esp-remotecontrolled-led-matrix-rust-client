use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
};

use esp_wlan_led_matrix_client::sync::Client;

fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");

    let text = std::env::var("TEXT");
    let text = text.as_deref().unwrap_or("hey!");

    let mut client = Client::connect(addr).expect("connection error");

    println!(
        "{} size {}x{} = {} pixels",
        addr,
        client.width(),
        client.height(),
        client.total_pixels()
    );

    client.fill(0, 0, 0).unwrap();
    Text::new(
        text,
        Point::new(0, 6),
        MonoTextStyle::new(&FONT_5X7, Rgb888::MAGENTA),
    )
    .draw(&mut client)
    .unwrap();
}
