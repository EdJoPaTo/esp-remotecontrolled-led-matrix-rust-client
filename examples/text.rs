use std::thread::sleep;
use std::time::Duration;

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
    let mut client = Client::connect(addr).expect("connection error");

    println!(
        "{} size {}x{} = {} pixels",
        addr,
        client.width(),
        client.height(),
        client.total_pixels()
    );

    let position = Point::new(0, 6);

    let text1 = Text::new(
        "Hey!",
        position,
        MonoTextStyle::new(&FONT_5X7, Rgb888::MAGENTA),
    );
    let text2 = Text::new(
        "there!",
        position,
        MonoTextStyle::new(&FONT_5X7, Rgb888::CYAN),
    );

    loop {
        client.fill(0, 0, 0).unwrap();
        text1.draw(&mut client).unwrap();
        client.flush().unwrap();
        sleep(Duration::from_secs(1));

        client.fill(0, 0, 0).unwrap();
        text2.draw(&mut client).unwrap();
        client.flush().unwrap();
        sleep(Duration::from_secs(1));
    }
}
