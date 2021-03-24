#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    // Each snake has fields representing the squares each section is on, an snake_array representing
    // whether the snake has a body part on that square, the health the snake is at, and the
    // number of sections queued for addition.
    pub snakes: Vec<Snake>,
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
            prev_healths.push(snake.health);
            tail_pos.push(snake.positions[snake.positions.len() - 1]);
            hit_inaccessible.push(false);
            was_queued.push(snake.queued > 0);
            eaten_apples.push(false);

            // skip if dead
            if snake.health == 0 {
                continue;
            }

            // get new head position
            let direction = directions[snake_idx];
            let new_head_signed = match direction {
                0 => snake.positions[0] as i16 + self.width as i16,
                1 => snake.positions[0] as i16 + 1,
                2 => snake.positions[0] as i16 - self.width as i16,
                3 => snake.positions[0] as i16 - 1,
                _ => panic!("Invalid direction"),
            };
            let new_head = if new_head_signed < 0
                || new_head_signed >= self.width as i16 * self.height as i16
                || (new_head_signed % self.width as i16 == 0 && direction == 1)
                || ((new_head_signed + 1) % self.width as i16 == 0 && direction == 3)
            {
                snake.health = 0;
                hit_inaccessible[snake_idx] = true;
                continue;
            } else {
                new_head_signed as u16
            };

            // move snake
            snake.positions.insert(0, new_head);
            if snake.queued == 0 {
                let last = snake.positions.pop().unwrap();
                if snake.positions[1..].iter().any(|pos| *pos == new_head) {
                    snake.positions.push(last);
                    snake.positions.remove(0);
                    hit_inaccessible[snake_idx] = true;
                    snake.health = 0;
                    continue;
                }
                snake.snake_arr[last as usize] = false;
            } else {
                if snake.positions[1..].iter().any(|pos| *pos == new_head) {
                    snake.positions.remove(0);
                    hit_inaccessible[snake_idx] = true;
                    snake.health = 0;
                    continue;
                }
                snake.queued -= 1;
            }
            snake.snake_arr[new_head as usize] = true;
            snake.health -= 1;

            // eat apple if available
            if self.apples[new_head as usize] {
                snake.health = 100;
                snake.queued += 1;
                self.apples[new_head as usize] = false;
                eaten_apples[snake_idx] = true;
            }
        }

        // snake-to-snake collisions
        for snake_idx in 0..self.snakes.len() {
            if self.snakes[snake_idx].health == 0 {
                continue;
            }
            for collide_snake_idx in 0..self.snakes.len() {
                if collide_snake_idx == snake_idx || self.snakes[collide_snake_idx].health == 0 {
                    continue;
                }
                if self.snakes[snake_idx].positions[0]
                    == self.snakes[collide_snake_idx].positions[0]
                {
                    if self.snakes[snake_idx].positions.len()
                        == self.snakes[collide_snake_idx].positions.len()
                    {
                        self.snakes[snake_idx].health = 0;
                        self.snakes[collide_snake_idx].health = 0;
                    } else if self.snakes[snake_idx].positions.len()
                        < self.snakes[collide_snake_idx].positions.len()
                    {
                        self.snakes[snake_idx].health = 0;
                    }
                } else if self.snakes[collide_snake_idx].positions[1..]
                    .iter()
                    .any(|pos| *pos == self.snakes[snake_idx].positions[0])
                {
                    self.snakes[snake_idx].health = 0;
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
            snake.health = prev_state.prev_healths[snake_idx];
            if prev_state.hit_inaccessible[snake_idx] {
                continue;
            }
            snake.snake_arr[snake.positions[0] as usize] = false;
            snake.snake_arr[prev_state.tail_pos[snake_idx] as usize] = true;
            let head = snake.positions[0];
            snake.positions.remove(0);
            if !prev_state.was_queued[snake_idx] {
                snake.positions.push(prev_state.tail_pos[snake_idx]);
            } else {
                snake.queued += 1;
            }
            if prev_state.eaten_apples[snake_idx] {
                self.apples[head as usize] = true;
                snake.queued -= 1;
            }
        }
    }

    pub fn add_snake(&mut self, positions: Vec<u16>, snake_arr: Vec<bool>, health: u8, queued: u8) {
        self.snakes.push(Snake {
            positions,
            snake_arr,
            health,
            queued,
        });
    }

    pub fn add_start_snake(&mut self, position: u16) {
        let mut snake_arr = vec![false; self.width as usize * self.height as usize];
        snake_arr[position as usize] = true;
        self.snakes.push(Snake {
            positions: vec![position],
            snake_arr,
            health: 100,
            queued: 2,
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snake {
    pub positions: Vec<u16>,
    pub snake_arr: Vec<bool>,
    pub health: u8,
    pub queued: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
        game.snakes[0].queued = 10;
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

        // eating apple
        let mut game = Game::new(7, 7);
        game.add_start_snake(0);
        game.add_start_snake(6);
        let mut apples = vec![false; 121];
        apples[1] = true;
        game.apples = apples;
        let game_clone = game.clone();
        let prev_state = game.move_snakes(&vec![1, 0]);
        game.unmove_snake(&prev_state);
        assert_eq!(game, game_clone);
    }
}
