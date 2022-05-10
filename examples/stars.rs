use std::time::Duration;

use esp_wlan_led_matrix_client::async_tokio::Client;
use rand::Rng;
use tokio::task;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let addr = std::env::var("ADDR");
    let addr = addr.as_deref().unwrap_or("espPixelmatrix:1337");
    let client = Client::connect(addr).await.expect("failed to connect");

    loop {
        let dur = match spawn_star(client.clone()).await {
            Ok(_) => {
                let dur = rand::thread_rng().gen_range(0..40_u64);
                let dur = dur.pow(2);
                Duration::from_millis(dur)
            }
            Err(err) => {
                eprintln!("spawn_star ERROR {}", err);
                Duration::from_secs(2)
            }
        };
        sleep(dur).await;
    }
}

async fn spawn_star(mut client: Client) -> std::io::Result<()> {
    async fn fade_away(mut client: Client, x: u8, y: u8) -> std::io::Result<()> {
        for bri in [100_u8, 0] {
            sleep(Duration::from_millis(150)).await;
            client.pixel(x, y, bri, bri, bri).await?;
            client.flush().await?;
        }
        Ok(())
    }

    let (x, y) = {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..client.width());
        let y = rng.gen_range(0..client.height());
        (x, y)
    };
    println!("star {:3} {:3}", x, y);

    client.pixel(x, y, 255, 255, 255).await?;
    client.flush().await?;

    task::spawn(async move {
        if let Err(err) = fade_away(client, x, y).await {
            println!("spawn_star ERROR {}", err);
        }
    });
    Ok(())
}
