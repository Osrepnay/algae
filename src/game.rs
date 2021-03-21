#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    // Each snake has fields representing the squares each section is on, an array representing
    // whether the snake has a body part on that square, the health the snake is at, and the
    // number of sections queued for addition.
    pub snakes: Vec<(Vec<u16>, Vec<bool>, u8, u8)>,
    pub apples: Vec<bool>,
    pub width: u8,
    pub height: u8,
}

impl Game {
    pub fn new(width: u8, height: u8) -> Game {
        Game {
            snakes: Vec::new(),
            apples: vec![false; width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn move_snakes(&mut self, directions: &Vec<u8>) -> ChangedState {
        let mut prev_healths = Vec::new();
        let mut tail_pos = Vec::new();
        let mut hit_inaccessible = Vec::new();
        let mut was_queued = Vec::new();
        let mut eaten_apples = Vec::new();
        for snake_idx in 0..self.snakes.len() {
            let snake = &mut self.snakes[snake_idx];

            // store for unmove
            prev_healths.push(snake.2);
            tail_pos.push(snake.0[snake.0.len() - 1]);
            hit_inaccessible.push(false);
            was_queued.push(snake.3 > 0);
            eaten_apples.push(false);

            // skip if dead
            if snake.2 == 0 {
                continue;
            }

            // get new head position
            let direction = directions[snake_idx];
            let new_head_signed = match direction {
                0 => snake.0[0] as i16 + self.width as i16,
                1 => snake.0[0] as i16 + 1,
                2 => snake.0[0] as i16 - self.width as i16,
                3 => snake.0[0] as i16 - 1,
                _ => panic!("Invalid direction"),
            };
            let new_head = if new_head_signed < 0
                || new_head_signed >= self.width as i16 * self.height as i16
                || (new_head_signed % self.width as i16 == 0 && direction == 1)
                || ((new_head_signed + 1) % self.width as i16 == 0 && direction == 3)
            {
                snake.2 = 0;
                hit_inaccessible[snake_idx] = true;
                continue;
            } else {
                new_head_signed as u16
            };

            // move snake
            snake.0.insert(0, new_head);
            if snake.3 == 0 {
                let last = snake.0.pop().unwrap();
                if snake.0[1..].iter().any(|pos| *pos == new_head) {
                    snake.0.push(last);
                    snake.0.remove(0);
                    hit_inaccessible[snake_idx] = true;
                    snake.2 = 0;
                    continue;
                }
                snake.1[last as usize] = false;
            } else {
                if snake.0[1..].iter().any(|pos| *pos == new_head) {
                    snake.0.remove(0);
                    hit_inaccessible[snake_idx] = true;
                    snake.2 = 0;
                    continue;
                }
                snake.3 -= 1;
            }
            snake.1[new_head as usize] = true;
            snake.2 -= 1;

            // eat apple if available
            if self.apples[new_head as usize] {
                snake.2 = 100;
                snake.3 += 1;
                self.apples[new_head as usize] = false;
                eaten_apples[snake_idx] = true;
            }
        }

        // snake-to-snake collisions
        for snake_idx in 0..self.snakes.len() {
            if self.snakes[snake_idx].2 == 0 {
                continue;
            }
            for collide_snake_idx in 0..self.snakes.len() {
                if collide_snake_idx == snake_idx || self.snakes[collide_snake_idx].2 == 0 {
                    continue;
                }
                if self.snakes[snake_idx].0[0] == self.snakes[collide_snake_idx].0[0] {
                    if self.snakes[snake_idx].0.len() == self.snakes[collide_snake_idx].0.len() {
                        self.snakes[snake_idx].2 = 0;
                        self.snakes[collide_snake_idx].2 = 0;
                    } else if self.snakes[snake_idx].0.len()
                        < self.snakes[collide_snake_idx].0.len()
                    {
                        self.snakes[snake_idx].2 = 0;
                    }
                } else if self.snakes[collide_snake_idx].0[1..]
                    .iter()
                    .any(|pos| *pos == self.snakes[snake_idx].0[0])
                {
                    self.snakes[snake_idx].2 = 0;
                }
            }
        }
        ChangedState {
            prev_healths,
            tail_pos,
            hit_inaccessible,
            was_queued,
            eaten_apples,
        }
    }

    pub fn unmove_snake(&mut self, prev_state: &ChangedState) {
        for snake_idx in 0..self.snakes.len() {
            let snake = &mut self.snakes[snake_idx];
            if prev_state.prev_healths[snake_idx] == 0 {
                continue;
            }
            snake.2 = prev_state.prev_healths[snake_idx];
            if prev_state.hit_inaccessible[snake_idx] {
                continue;
            }
            snake.1[snake.0[0] as usize] = false;
            snake.1[prev_state.tail_pos[snake_idx] as usize] = true;
            snake.0.remove(0);
            if !prev_state.was_queued[snake_idx] {
                snake.0.push(prev_state.tail_pos[snake_idx]);
            }
            if prev_state.eaten_apples[snake_idx] {
                snake.2 -= 1;
            }
            if prev_state.was_queued[snake_idx] {
                snake.3 += 1;
            }
        }
    }

    pub fn add_snake(&mut self, positions: Vec<u16>, snake_arr: Vec<bool>, health: u8, queued: u8) {
        self.snakes.push((positions, snake_arr, health, queued));
    }

    pub fn add_start_snake(&mut self, position: u16) {
        let mut snake_arr = vec![false; self.width as usize * self.height as usize];
        snake_arr[position as usize] = true;
        self.snakes.push((vec![position], snake_arr, 100, 2));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedState {
    prev_healths: Vec<u8>,
    tail_pos: Vec<u16>,
    hit_inaccessible: Vec<bool>,
    was_queued: Vec<bool>,
    eaten_apples: Vec<bool>,
}

mod test {
    // Rust says that the import is unused for some reason?
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_unmake_move() {
        // wall collisions
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(6);
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![0, 1]);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);

        // head-to-head collisions
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(2);
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![1, 3]);
        println!("{:?}", game);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);

        // snake-to-snake body collisions
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(8);
        game.move_snakes(&vec![0, 0]);
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![1, 0]);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);

        // self collisions
        let mut game = Game::new(7, 7);
        game.add_start_snake(1);
        game.snakes[0].3 = 10;
        game.add_start_snake(6);
        game.move_snakes(&vec![0, 0]);
        game.move_snakes(&vec![3, 0]);
        game.move_snakes(&vec![2, 0]);
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![1, 0]);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(6);
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![2, 0]);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);
    }
}
