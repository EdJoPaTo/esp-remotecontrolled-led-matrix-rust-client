use std::thread::sleep;
use std::time::Duration;

use bracket_color::prelude::HSV;
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

struct Directions {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl Directions {
    fn new(left: bool, right: bool, up: bool, down: bool) -> Self {
        Self { left, right, up, down }
    }

    fn join(first: Directions, second: Directions) -> Self {
        Self {
            first.left && second.left,
            first.right && second.right,
            first.up && second.up,
            first.down && second.down,
        }
    }

    // fn one(directions: Directions) -> (bool) {
    //     let result =
    //     directions.left && !directions.right && !directions.up && !directions.down ||
    //     !directions.left && directions.right && !directions.up && !directions.down ||
    //     !directions.left && !directions.right && directions.up && !directions.down ||
    //     !directions.left && !directions.right && !directions.up && directions.down
    //     (result)
    // }
    fn count(directions: Directions) -> (u8) {
        let result = {
            if directions.left{
                result += 1
            }
            if directions.right{
                result += 1
            }
            if directions.up{
                result += 1
            }
            if directions.down{
                result += 1
            }
        }
        (result)
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

#[allow(clippy::too_many_lines)]
fn snake(client: &mut Client) -> std::io::Result<()> {
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

        // let previous_priority =
        // Directions::new(
        //     start.x > food.x,
        //     start.x < food.x,
        //     false,
        //     false,
        // )
        // let prio = true;
        let previous = {
            if snake[0].x < snake[1].x {
                "l"
            } else if snake[0].x > snake[1].x {
                "r";
            }
        };

        loop {
            let next_point = {
                let head = &snake[0];
                let neck = &snake[1];
                // Directions wont leave the area. Saturating sub prevents < 0, min() prevents > width/height
                let left = Point::new(head.x.saturating_sub(1), head.y);
                let right = Point::new(head.x.max(width - 1), head.y);
                let up = Point::new(head.x, head.y.saturating_sub(1));
                let down = Point::new(head.x, head.y.max(height - 1));

                let possible =
                Directions::new(
                    !snake.contains(&left),
                    !snake.contains(&right),
                    !snake.contains(&up),
                    !snake.contains(&down),
                );
                let desired =
                Directions::new(
                    head.x > food.x,
                    head.x < food.x,
                    head.y > food.y,
                    head.y < food.y,
                );
                // let priority =
                // Directions::new(
                //     head.y == food.y && head.x > food.x || prio && start.x > food.x,
                //     head.y == food.y && head.x < food.x || prio && start.x < food.x,
                //     head.x == food.x && head.y > food.y,
                //     head.x == food.x && head.y < food.y,
                // );
                let priority =
                Directions::new(
                    head.y == food.y && head.x > food.x,// || prio && start.x > food.x,
                    head.y == food.y && head.x < food.x,// || prio && start.x < food.x,
                    head.x == food.x && head.y > food.y,
                    head.x == food.x && head.y < food.y,
                );
                //prio = false;
                previous = {
                    if head.x < neck.x && !previous.starts_with('l') {
                        "l{}", previous
                    } else if head.x > neck.x && !previous.starts_with('r') {
                        "r{}", previous
                    } else if head.y < neck.y && !previous.starts_with('u') {
                        "u{}", previous
                    } else if head.y > neck.y && !previous.starts_with('d') {
                        "d{}", previous
                    }
                };

                let possible_desired =
                Directions::join(
                    possible, desired
                );
                // let success = {
                //     Directions::one(
                //         possible_desired
                //     )
                // };

                let fast_success = Directions::count(possible_desired);
                // if success {
                if fast_success == 1 {
                    if possible_desired.left{
                        left
                    } else if possible_desired.right{
                        right
                    } else if possible_desired.up{
                        up
                    } else if possible_desired.down{
                        down
                    }
                    else {
                        println!("check ur code! #1");
                    }
                }
                else if fast_success > 1 {
                    let possible_desired_priority =
                    Directions::join(
                        possible_desired, priority
                    );
                    let normal_success = Directions::count(possible_desired_priority);
                    if normal_success == 1 {
                        if possible_desired_priority.left{
                            left
                        } else if possible_desired_priority.right{
                            right
                        } else if possible_desired_priority.up{
                            up
                        } else if possible_desired_priority.down{
                            down
                        }
                        else {
                            println!("check ur code! #2");
                        }
                    } else{
                        println!("check ur code! #3");
                    }
                } else{
                    if previous[0] == 'l' && possible.left{
                        left
                    } else if previous[0] == 'r' && possible.right{
                        right
                    } else if previous[0] == 'u' && possible.up{
                        up
                    } else if previous[0] == 'd' && possible.down{
                        down
                    } else if previous[1] == 'l' && possible.left{
                        left
                    } else if previous[1] == 'r' && possible.right{
                        right
                    } else if previous[1] == 'u' && possible.up{
                        up
                    } else if previous[1] == 'd' && possible.down{
                        down
                    } else if previous[2] == 'l' && possible.left{
                        left
                    } else if previous[2] == 'r' && possible.right{
                        right
                    } else if previous[2] == 'u' && possible.up{
                        up
                    } else if previous[2] == 'd' && possible.down{
                        down
                    } else if previous[3] == 'l' && possible.left{
                        left
                    } else if previous[3] == 'r' && possible.right{
                        right
                    } else if previous[3] == 'u' && possible.up{
                        up
                    } else if previous[3] == 'd' && possible.down{
                        down
                    } else{
                        println!("check ur code! #4");
                    }
                }

                // #[allow(clippy::if_not_else, clippy::collapsible_else_if)]
                // if head.x > food.x {
                //     if !snake.contains(&left) {
                //         left
                //     } else if !snake.contains(&up) {
                //         up
                //     } else if !snake.contains(&down) {
                //         down
                //     } else {
                //         right
                //     }
                // } else if head.x < food.x {
                //     if !snake.contains(&right) {
                //         right
                //     } else if !snake.contains(&down) {
                //         down
                //     } else if !snake.contains(&up) {
                //         up
                //     } else {
                //         left
                //     }
                // } else if head.y > food.y {
                //     if !snake.contains(&up) {
                //         up
                //     } else if !snake.contains(&left) {
                //         left
                //     } else if !snake.contains(&right) {
                //         right
                //     } else {
                //         down
                //     }
                // } else {
                //     if !snake.contains(&down) {
                //         down
                //     } else if !snake.contains(&right) {
                //         right
                //     } else if !snake.contains(&left) {
                //         left
                //     } else {
                //         up
                //     }
                // }
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

            // Hit itself
            if snake.contains(&next_point) {
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
                break;
            }

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
