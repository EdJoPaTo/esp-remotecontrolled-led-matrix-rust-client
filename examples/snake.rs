use std::thread::sleep;
use std::time::Duration;

use esp_wlan_led_matrix_client::sync::Client;
use rand::Rng;

const RUN_SLEEP: Duration = Duration::from_millis(200);
const DECAY_SLEEP: Duration = Duration::from_millis(100);

#[derive(Debug, PartialEq)]
struct Point {
    x: u8,
    y: u8,
}

impl Point {
    fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    fn random(width: u8, height: u8) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..width - 1);
        let y = rng.gen_range(0..height - 1);
        Self { x, y }
    }
}

struct Gamestate {
    snake: Vec<Point>,
    food: Point,
    color: (u8, u8, u8),
}

impl Gamestate {
    fn new(width: u8, height: u8) -> Self {
        let food = Point::random(width, height);

        let start = Point::new(width / 2, height / 2);
        let end = {
            let x = if start.x < food.x {
                start.x - 1
            } else {
                start.x + 1
            };
            Point::new(x, start.y)
        };

        let r = rand::random::<u8>();
        let g = rand::random::<u8>();
        let b = rand::random::<u8>();

        Self {
            snake: vec![start, end],
            food,
            color: (r, g, b),
        }
    }
}

fn main() {
    let addr = "espPixelmatrix:1337";

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

fn snake(client: &mut Client) -> std::io::Result<()> {
    let width = client.width();
    let height = client.height();
    loop {
        let mut state = Gamestate::new(width, height);

        loop {
            let next_point = {
                let start = &state.snake[0];
                let left = Point::new(start.x - 1, start.y);
                let right = Point::new(start.x + 1, start.y);
                let up = Point::new(start.x, start.y - 1);
                let down = Point::new(start.x, start.y + 1);
                #[allow(clippy::if_not_else, clippy::collapsible_else_if)]
                if start.x > state.food.x {
                    if !state.snake.contains(&left) {
                        left
                    } else if !state.snake.contains(&up) {
                        up
                    } else if !state.snake.contains(&down) {
                        down
                    } else {
                        right
                    }
                } else if start.x < state.food.x {
                    if !state.snake.contains(&right) {
                        right
                    } else if !state.snake.contains(&down) {
                        down
                    } else if !state.snake.contains(&up) {
                        up
                    } else {
                        left
                    }
                } else if start.y > state.food.y {
                    if !state.snake.contains(&up) {
                        up
                    } else if !state.snake.contains(&left) {
                        left
                    } else if !state.snake.contains(&right) {
                        right
                    } else {
                        down
                    }
                } else {
                    if !state.snake.contains(&down) {
                        down
                    } else if !state.snake.contains(&right) {
                        right
                    } else if !state.snake.contains(&left) {
                        left
                    } else {
                        up
                    }
                }
            };

            #[cfg(debug_assertions)]
            println!(
                "snake goes to {:3} {:3}  food is at {:3} {:3}",
                next_point.x, next_point.y, state.food.x, state.food.y
            );

            if state.snake.contains(&next_point) {
                for point in state.snake {
                    client.pixel(point.x, point.y, 0, 0, 0)?;
                    client.flush()?;
                    sleep(DECAY_SLEEP);
                }

                client.pixel(state.food.x, state.food.y, 0, 0, 0)?;
                break;
            }

            if next_point == state.food {
                state.food = Point::random(width, height);
            } else {
                let last = state.snake.pop().unwrap();
                client.pixel(last.x, last.y, 0, 0, 0)?;
            }

            client.pixel(
                next_point.x,
                next_point.y,
                state.color.0,
                state.color.1,
                state.color.2,
            )?;
            state.snake.insert(0, next_point);

            client.pixel(
                state.food.x,
                state.food.y,
                255 - state.color.0,
                255 - state.color.1,
                255 - state.color.2,
            )?;

            client.flush()?;
            sleep(RUN_SLEEP);
        }
    }
}
