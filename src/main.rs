pub mod algae;
pub mod game;

use game::Game;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use warp::http::StatusCode;
use warp::Filter;
use warp::Rejection;

#[tokio::main]
async fn main() {
    let index = warp::path::end().map(|| {
        warp::reply::json(&json!({
            "apiversion": "1",
            "color": "#FF0000",
            "head": "safe",
            "tail": "block-bum",
        }))
    });
    // darn iot coffeemakers these days
    let start = warp::path("start")
        .and(warp::post())
        .map(|| warp::reply::with_status("", StatusCode::IM_A_TEAPOT));
    let end = warp::path("end")
        .and(warp::post())
        .map(|| warp::reply::with_status("", StatusCode::IM_A_TEAPOT));
    let get_move = warp::path("move")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(|sent_move: Move| async move {
            let start = Instant::now();
            println!("request: {:?}", sent_move);
            let mut game = Game::new(sent_move.board.width, sent_move.board.height);
            for apple in sent_move.board.food {
                let x = apple.get("x").unwrap();
                let y = apple.get("y").unwrap();
                game.apples[(y * sent_move.board.width as u16 + x) as usize] = true;
            }
            let mut my_positions: Vec<u16> = Vec::new();
            let mut my_snake_arr =
                vec![false; sent_move.board.width as usize * sent_move.board.height as usize];
            let mut my_queued = 0;
            for pos_idx in 0..sent_move.you.body.len() {
                let x = sent_move.you.body[pos_idx].get("x").unwrap();
                let y = sent_move.you.body[pos_idx].get("y").unwrap();
                let pos = y * sent_move.board.width as u16 + x;
                if my_positions.len() > 0 && my_positions[my_positions.len() - 1] == pos {
                    my_queued += 1;
                } else {
                    my_positions.push(pos);
                    my_snake_arr[pos as usize] = true;
                }
            }
            game.add_snake(my_positions, my_snake_arr, sent_move.you.health, my_queued);
            for snake in sent_move.board.snakes {
                if snake == sent_move.you {
                    continue;
                }
                let mut positions: Vec<u16> = Vec::new();
                let mut snake_arr =
                    vec![false; sent_move.board.width as usize * sent_move.board.height as usize];
                let mut queued = 0;
                for pos_idx in 0..snake.body.len() {
                    let x = snake.body[pos_idx].get("x").unwrap();
                    let y = snake.body[pos_idx].get("y").unwrap();
                    let pos = y * sent_move.board.width as u16 + x;
                    if positions.len() > 0 && positions[positions.len() - 1] == pos {
                        queued += 1;
                    } else {
                        positions.push(pos);
                        snake_arr[pos as usize] = true;
                    }
                }
                game.add_snake(positions, snake_arr, snake.health, queued);
            }
            println!("game: {:?}", game);

            let mut depth = 1;
            let mut best_move = 0;
            // subtract ms to avoid accidentally taking slightly too long
            while start.elapsed().as_millis() < sent_move.game.timeout - 375 {
                let best_move_temp = algae::best_move(
                    &mut game,
                    depth,
                    (sent_move.game.timeout - start.elapsed().as_millis() - 375) as i128,
                );
                match best_move_temp {
                    Some(best_move_temp) => best_move = best_move_temp.0,
                    None => break,
                }
                depth += 1;
            }
            println!("depth: {}, move: {}", depth, best_move);
            let move_int_to_str = ["up", "right", "down", "left"];
            Ok(warp::reply::json(&json!({
                "move": move_int_to_str[best_move as usize],
                "shout": "*aggressively yells*"
            }))) as Result<_, Rejection>
        });
    let routes = index
        .or(start)
        .or(end)
        .or(get_move)
        .with(warp::log("status_log"));
    let port = std::env::var("PORT")
        .expect("PORT Environment Variable not set")
        .parse()
        .expect("PORT is not a valid port number");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

#[derive(Debug, Deserialize)]
struct Move {
    game: SentGame,
    turn: u32,
    board: Board,
    you: Battlesnake,
}

#[derive(Debug, Deserialize)]
struct SentGame {
    id: String,
    timeout: u128,
}

#[derive(Debug, Deserialize)]
struct Board {
    height: u8,
    width: u8,
    food: Vec<HashMap<String, u16>>,
    hazards: Vec<HashMap<String, u16>>,
    snakes: Vec<Battlesnake>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Battlesnake {
    id: String,
    name: String,
    health: u8,
    body: Vec<HashMap<String, u16>>,
    latency: String,
    head: HashMap<String, u16>,
    length: u16,
    shout: String,
}
