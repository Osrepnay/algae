use crate::game::Game;
use std::time::Instant;

pub fn best_move(game: &mut Game, depth: u8, search_time: i128) -> Option<(u8, f64)> {
    let start = Instant::now();
    if search_time < 0 {
        return None;
    }
    let mut best_move = (0, f64::NEG_INFINITY);
    for direction in 0..4 {
        if game.snakes[0].positions.len() == 1 && direction == 2 {
            continue;
        }
        let new_head_signed = match direction {
            0 => game.snakes[0].positions[0] as i16 + game.width as i16,
            1 => game.snakes[0].positions[0] as i16 + 1,
            2 => game.snakes[0].positions[0] as i16 - game.width as i16,
            3 => game.snakes[0].positions[0] as i16 - 1,
            _ => panic!("Invalid direction"),
        };
        if new_head_signed < 0
            || new_head_signed >= game.width as i16 * game.height as i16
            || (new_head_signed % game.width as i16 == 0 && direction == 1)
            || ((new_head_signed + 1) % game.width as i16 == 0 && direction == 3)
        {
            continue;
        }
        if game.snakes[0].positions[1..game.snakes[0].positions.len() - 1]
            .iter()
            .any(|pos| *pos == new_head_signed as u16)
        {
            continue;
        }
        let score = min(
            game,
            direction,
            best_move.1,
            f64::INFINITY,
            depth,
            search_time - start.elapsed().as_millis() as i128,
        )?;
        if score > best_move.1 {
            best_move.0 = direction;
            best_move.1 = score;
        }
    }
    Some(best_move)
}

pub fn max(
    game: &mut Game,
    mut alpha: f64,
    beta: f64,
    depth: u8,
    search_time: i128,
) -> Option<f64> {
    let start = Instant::now();
    if search_time < 0 {
        return None;
    }
    if game.snakes[0].health == 0 {
        return Some(-10000.0);
    }
    if depth <= 0 {
        return Some(eval(game));
    }
    for direction in 0..4 {
        if game.snakes[0].positions.len() == 1 && direction == 2 {
            continue;
        }
        let new_head_signed = match direction {
            0 => game.snakes[0].positions[0] as i16 + game.width as i16,
            1 => game.snakes[0].positions[0] as i16 + 1,
            2 => game.snakes[0].positions[0] as i16 - game.width as i16,
            3 => game.snakes[0].positions[0] as i16 - 1,
            _ => panic!("Invalid direction"),
        };
        if new_head_signed < 0
            || new_head_signed >= game.width as i16 * game.height as i16
            || (new_head_signed % game.width as i16 == 0 && direction == 1)
            || ((new_head_signed + 1) % game.width as i16 == 0 && direction == 3)
        {
            continue;
        }
        if game.snakes[0].positions[1..game.snakes[0].positions.len() - 1]
            .iter()
            .any(|pos| *pos == new_head_signed as u16)
        {
            continue;
        }
        let score = min(
            game,
            direction,
            alpha,
            beta,
            depth,
            search_time - start.elapsed().as_millis() as i128,
        )?;
        if score >= beta {
            return Some(beta);
        }
        if score > alpha {
            alpha = score;
        }
    }
    Some(alpha)
}

pub fn min(
    game: &mut Game,
    own_snake_move: u8,
    alpha: f64,
    mut beta: f64,
    depth: u8,
    search_time: i128,
) -> Option<f64> {
    let start = Instant::now();
    if search_time < 0 {
        return None;
    }
    if !game.snakes[1..].iter().any(|snake| snake.health > 0) {
        return Some(10000.0);
    }
    if depth <= 0 {
        return Some(eval(game));
    }
    if game.snakes[1].health <= 0 {
        let score = min_rec(
            game,
            &mut vec![own_snake_move, 0],
            alpha,
            beta,
            depth,
            search_time - start.elapsed().as_millis() as i128,
        )?;
        if score <= alpha {
            return Some(alpha);
        }
        if score < beta {
            beta = score;
        }
        return Some(beta);
    }
    for direction in 0..4 {
        if game.snakes[1].positions.len() == 1 && direction == 2 {
            continue;
        }
        let new_head_signed = match direction {
            0 => game.snakes[1].positions[0] as i16 + game.width as i16,
            1 => game.snakes[1].positions[0] as i16 + 1,
            2 => game.snakes[1].positions[0] as i16 - game.width as i16,
            3 => game.snakes[1].positions[0] as i16 - 1,
            _ => panic!("Invalid direction"),
        };
        if new_head_signed < 0
            || new_head_signed >= game.width as i16 * game.height as i16
            || (new_head_signed % game.width as i16 == 0 && direction == 1)
            || ((new_head_signed + 1) % game.width as i16 == 0 && direction == 3)
        {
            continue;
        }
        if game.snakes[1].positions[1..game.snakes[1].positions.len() - 1]
            .iter()
            .any(|pos| *pos == new_head_signed as u16)
        {
            continue;
        }
        let score = min_rec(
            game,
            &mut vec![own_snake_move, direction],
            alpha,
            beta,
            depth,
            search_time - start.elapsed().as_millis() as i128,
        )?;
        if score <= alpha {
            return Some(alpha);
        }
        if score < beta {
            beta = score;
        }
    }
    Some(beta)
}

fn min_rec(
    game: &mut Game,
    other_snake_moves: &mut Vec<u8>,
    alpha: f64,
    mut beta: f64,
    depth: u8,
    search_time: i128,
) -> Option<f64> {
    let start = Instant::now();
    if search_time < 0 {
        return None;
    }
    if other_snake_moves.len() == game.snakes.len() {
        let prev_state = game.move_snakes(other_snake_moves);
        let score = max(
            game,
            alpha,
            beta,
            depth - 1,
            search_time - start.elapsed().as_millis() as i128,
        )?;
        game.unmove_snake(&prev_state);
        if score <= alpha {
            return Some(alpha);
        }
        if score < beta {
            beta = score;
        }
    } else {
        if game.snakes[other_snake_moves.len()].health == 0 {
            other_snake_moves.push(0);
            let score = min_rec(
                game,
                other_snake_moves,
                alpha,
                beta,
                depth,
                search_time - start.elapsed().as_millis() as i128,
            )?;
            other_snake_moves.pop();
            if score <= alpha {
                return Some(alpha);
            }
            if score < beta {
                beta = score;
            }
            return Some(beta);
        }
        for direction in 0..4 {
            if game.snakes[other_snake_moves.len()].positions.len() == 1 && direction == 2 {
                continue;
            }
            let new_head_signed = match direction {
                0 => game.snakes[other_snake_moves.len()].positions[0] as i16 + game.width as i16,
                1 => game.snakes[other_snake_moves.len()].positions[0] as i16 + 1,
                2 => game.snakes[other_snake_moves.len()].positions[0] as i16 - game.width as i16,
                3 => game.snakes[other_snake_moves.len()].positions[0] as i16 - 1,
                _ => panic!("Invalid direction"),
            };
            if new_head_signed < 0
                || new_head_signed >= game.width as i16 * game.height as i16
                || (new_head_signed % game.width as i16 == 0 && direction == 1)
                || ((new_head_signed + 1) % game.width as i16 == 0 && direction == 3)
            {
                continue;
            }
            if game.snakes[other_snake_moves.len()].positions
                [1..game.snakes[other_snake_moves.len()].positions.len() - 1]
                .iter()
                .any(|pos| *pos == new_head_signed as u16)
            {
                continue;
            }
            other_snake_moves.push(direction);
            let score = min_rec(
                game,
                other_snake_moves,
                alpha,
                beta,
                depth,
                search_time - start.elapsed().as_millis() as i128,
            )?;
            other_snake_moves.pop();
            if score <= alpha {
                return Some(alpha);
            }
            if score < beta {
                beta = score;
            }
        }
    }
    Some(beta)
}

pub fn eval(game: &Game) -> f64 {
    fn cast_rays(idx: u16, all_blockers: &Vec<bool>, width: u8, height: u8) -> u16 {
        let mut total_size = 0;
        let mut counter = 1;
        while idx + counter * (width as u16) < width as u16 * height as u16
            && !all_blockers[(idx + counter * width as u16) as usize]
        {
            total_size += 1;
            counter += 1;
        }

        let mut counter = 1;
        while (idx + counter) % width as u16 != 0 && !all_blockers[(idx + counter) as usize] {
            total_size += 1;
            counter += 1;
        }

        let mut counter = 1;
        while idx as i16 - counter * width as i16 >= 0
            && !all_blockers[(idx as i16 - counter * width as i16) as usize]
        {
            total_size += 1;
            counter += 1;
        }

        let mut counter = 1;
        while (idx as i16 - counter + 1) % width as i16 != 0
            && !all_blockers[(idx as i16 - counter + 1) as usize]
        {
            total_size += 1;
            counter += 1;
        }
        total_size
    }

    let self_dead = game.snakes[0].health == 0;
    let others_dead = !game.snakes[1..].iter().any(|snake| snake.health > 0);
    if self_dead && others_dead {
        return 0.0;
    } else if self_dead {
        return -10000.0;
    } else if others_dead {
        return 10000.0;
    }
    let mut all_blockers = Vec::new();
    for square in 0..game.width as u16 * game.height as u16 {
        all_blockers.push(game.snakes.iter().any(|snake| snake.snake_arr[square as usize]));
    }
    let own_score = game.snakes[0].positions.len() as f64
        + game.snakes[0].queued as f64
        + cast_rays(game.snakes[0].positions[0], &all_blockers, game.width, game.height) as f64
            / (game.width as f64 + game.height as f64)
        + (game.snakes[0].health as f64 - 50.0) / 5.0;
    let mut other_score = 0.0;
    for other_snake in &game.snakes[1..] {
        if other_snake.health == 0 {
            continue;
        }
        other_score += other_snake.positions.len() as f64
            + other_snake.queued as f64
            + cast_rays(other_snake.positions[0], &all_blockers, game.width, game.height) as f64
                / (game.width as f64 + game.height as f64)
            + (other_snake.health as f64 - 50.0) / 5.0;
    }
    own_score - other_score / (game.snakes.len() - 1) as f64
}

mod test {
    // Rust says that the import is unused for some reason?
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_bestmove() {
        // self trap
        let mut game = Game::new(7, 7);
        game.add_start_snake(1);
        game.snakes[0].queued = 10;
        game.add_start_snake(6);
        game.move_snakes(&vec![0, 0]);
        game.move_snakes(&vec![3, 0]);
        game.move_snakes(&vec![2, 0]);
        assert_eq!(best_move(&mut game, 1, i128::MAX).unwrap().1, -10000.0);

        // trap the other snake
        let mut game = Game::new(7, 7);
        game.add_start_snake(9);
        game.snakes[0].queued = 3;
        game.add_start_snake(0);
        game.snakes[1].queued = 3;
        game.move_snakes(&vec![1, 1]);
        game.move_snakes(&vec![1, 1]);
        game.move_snakes(&vec![1, 1]);
        assert_eq!(best_move(&mut game, 2, i128::MAX).unwrap(), (2, 10000.0));

        // avoid losing head-to-head
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(6);
        game.snakes[1].queued = 3;
        game.move_snakes(&vec![1, 3]);
        game.move_snakes(&vec![1, 3]);
        game.move_snakes(&vec![1, 3]);
        let best_move = best_move(&mut game, 2, i128::MAX).unwrap().0;
        assert_ne!(best_move, 1);
        assert_ne!(best_move, 3);
    }
}
