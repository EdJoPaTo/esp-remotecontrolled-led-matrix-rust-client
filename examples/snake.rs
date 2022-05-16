use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

use bracket_color::prelude::HSV;
use esp_wlan_led_matrix_client::sync::Client;
use snake_logic::{get_next_point, Point};

const RUN_SLEEP: Duration = Duration::from_millis(200);
const DECAY_SLEEP: Duration = Duration::from_millis(100);

fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");

    loop {
        match Client::connect(addr) {
            Ok(mut client) => {
                println!(
                    "size {}x{} = {} pixels",
                    client.width(),
                    client.height(),
                    client.total_pixels()
                );

                if let Err(err) = snake(&mut client) {
                    eprintln!("ERROR: {}", err);
                }
            }
            Err(err) => {
                eprintln!("CONNECT ERROR: {}", err);
                sleep(Duration::from_millis(500));
            }
        }
    }
}

fn do_death<S: Read + Write>(
    client: &mut Client<S>,
    snake: &[Point],
    food: Point,
) -> std::io::Result<()> {
    println!(
        "snake length {:3} died at {:3} {:3}",
        snake.len(),
        snake.first().unwrap().x,
        snake.first().unwrap().y,
    );
    for point in snake {
        client.pixel(point.x, point.y, 0, 0, 0)?;
        client.flush()?;
        sleep(DECAY_SLEEP);
    }

    client.pixel(food.x, food.y, 0, 0, 0)?;
    Ok(())
}

fn snake<S: Read + Write>(client: &mut Client<S>) -> std::io::Result<()> {
    let width = client.width();
    let height = client.height();
    loop {
        let mut food = Point::random(width, height);
        let mut hue = rand::random::<f32>() % 360.0;

        let mut snake = {
            let start = Point::new(width / 2, height / 2);
            let end = {
                let x = if start.x < food.x {
                    start.x - 1
                } else {
                    start.x + 1
                };
                Point::new(x, start.y)
            };
            vec![start, end]
        };

        loop {
            let next_point = if let Some(point) = get_next_point(width, height, &snake, food) {
                // Hits itself
                if snake.contains(&point) {
                    do_death(client, &snake, food)?;
                    break;
                }

                point
            } else {
                do_death(client, &snake, food)?;
                break;
            };

            #[cfg(debug_assertions)]
            println!(
                "snake length {:3} goes to {:3} {:3}  food is at {:3} {:3}",
                snake.len(),
                next_point.x,
                next_point.y,
                food.x,
                food.y
            );

            if next_point == food {
                food = Point::random(width, height);
            } else {
                let last = snake.pop().unwrap();
                client.pixel(last.x, last.y, 0, 0, 0)?;
            }

            hue = (hue + 5.0) % 360.0;
            {
                let (r, g, b) = hue_to_rgb(hue);
                client.pixel(next_point.x, next_point.y, r, g, b)?;
            }
            snake.insert(0, next_point);

            {
                let (r, g, b) = hue_to_rgb((hue + 180.0) % 360.0);
                client.pixel(food.x, food.y, r, g, b)?;
            }

            client.flush()?;
            sleep(RUN_SLEEP);
        }
    }
}

/// Converts from f32 Hue to u8 rgb values
/// * `hue` - Hue from 0.0 to 360.0
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn hue_to_rgb(hue: f32) -> (u8, u8, u8) {
    let hsv = HSV::from_f32(hue / 360.0, 1.0, 1.0);
    let rgb = hsv.to_rgb();

    let red = (rgb.r * 255.0) as u8;
    let green = (rgb.g * 255.0) as u8;
    let blue = (rgb.b * 255.0) as u8;

    (red, green, blue)
}
