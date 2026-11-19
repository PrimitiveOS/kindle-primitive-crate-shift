//! Warehouse puzzles and move history shared by the Kindle and desktop front ends.

use serde::{Deserialize, Serialize};

pub const COLUMNS: usize = 8;
pub const ROWS: usize = 7;

pub const LEVELS: [Level; 8] = [
    Level {
        title: "First delivery",
        tiles: [
            "########", "#      #", "#   .  #", "#   $  #", "# @    #", "#      #", "########",
        ],
    },
    Level {
        title: "Twin docks",
        tiles: [
            "########", "# .  . #", "#      #", "# $$   #", "#  @   #", "#      #", "########",
        ],
    },
    Level {
        title: "Central shelf",
        tiles: [
            "########", "#  .   #", "# ## . #", "#   $$ #", "#  # @ #", "#      #", "########",
        ],
    },
    Level {
        title: "Side corridor",
        tiles: [
            "########", "#.     #", "# # #  #", "#   $$ #", "# # #@.#", "#      #", "########",
        ],
    },
    Level {
        title: "Receiving bay",
        tiles: [
            "########", "# . .  #", "# #  # #", "#  $$  #", "#.$    #", "#    @ #", "########",
        ],
    },
    Level {
        title: "Cross aisle",
        tiles: [
            "########", "#.  #. #", "#  $   #", "# # #  #", "# $$ # #", "#.   @ #", "########",
        ],
    },
    Level {
        title: "Last shipment",
        tiles: [
            "########", "# .. . #", "# # #  #", "# $ $  #", "#   #$ #", "# . $@ #", "########",
        ],
    },
    Level {
        title: "Staging floor",
        tiles: [
            "########", "#....  #", "# ##   #", "#  $$$ #", "#   $  #", "# @    #", "########",
        ],
    },
];

pub struct Level {
    pub title: &'static str,
    tiles: [&'static str; ROWS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Up, Self::Left, Self::Down, Self::Right];

    pub fn destination(self, column: usize, row: usize) -> Option<(usize, usize)> {
        let cell = match self {
            Self::Up => (column, row.checked_sub(1)?),
            Self::Left => (column.checked_sub(1)?, row),
            Self::Down => (column, row + 1),
            Self::Right => (column + 1, row),
        };
        (cell.0 < COLUMNS && cell.1 < ROWS).then_some(cell)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Dock,
    Crate,
    DockedCrate,
    Worker,
}

#[derive(Clone, Copy)]
struct Position {
    worker: (usize, usize),
    crates: u64,
    pushes: usize,
}

fn mask(column: usize, row: usize) -> u64 {
    1 << (row * COLUMNS + column)
}

pub struct Puzzle {
    walls: u64,
    docks: u64,
    position: Position,
    history: Vec<(Position, Direction)>,
}

impl Puzzle {
    fn new(level: &Level) -> Self {
        let mut puzzle = Self {
            walls: 0,
            docks: 0,
            position: Position {
                worker: (0, 0),
                crates: 0,
                pushes: 0,
            },
            history: Vec::new(),
        };
        for (row, line) in level.tiles.iter().enumerate() {
            assert_eq!(line.len(), COLUMNS);
            for (column, tile) in line.bytes().enumerate() {
                let spot = mask(column, row);
                match tile {
                    b'#' => puzzle.walls |= spot,
                    b'.' => puzzle.docks |= spot,
                    b'$' => puzzle.position.crates |= spot,
                    b'@' => puzzle.position.worker = (column, row),
                    b' ' => {}
                    _ => unreachable!("invalid warehouse tile"),
                }
            }
        }
        puzzle
    }

    pub fn tile(&self, column: usize, row: usize) -> Tile {
        let spot = mask(column, row);
        if self.walls & spot != 0 {
            Tile::Wall
        } else if (column, row) == self.position.worker {
            Tile::Worker
        } else {
            match (self.position.crates & spot != 0, self.docks & spot != 0) {
                (true, true) => Tile::DockedCrate,
                (true, false) => Tile::Crate,
                (false, true) => Tile::Dock,
                (false, false) => Tile::Floor,
            }
        }
    }

    pub fn worker(&self) -> (usize, usize) {
        self.position.worker
    }

    pub fn moves(&self) -> usize {
        self.history.len()
    }

    pub fn pushes(&self) -> usize {
        self.position.pushes
    }

    pub fn complete(&self) -> bool {
        self.position.crates == self.docks
    }

    pub fn push(&mut self, direction: Direction) -> bool {
        if self.complete() {
            return false;
        }
        let (x, y) = self.position.worker;
        let Some((next_x, next_y)) = direction.destination(x, y) else {
            return false;
        };
        let origin = mask(next_x, next_y);
        if self.walls & origin != 0 {
            return false;
        }
        let mut next = self.position;
        if next.crates & origin != 0 {
            let Some((x, y)) = direction.destination(next_x, next_y) else {
                return false;
            };
            let target = mask(x, y);
            if (self.walls | self.position.crates) & target != 0 {
                return false;
            }
            next.crates ^= origin | target;
            next.pushes += 1;
        }
        next.worker = (next_x, next_y);
        self.history.push((self.position, direction));
        self.position = next;
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some((position, _)) = self.history.pop() else {
            return false;
        };
        self.position = position;
        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    active: usize,
    moves: [Vec<Direction>; LEVELS.len()],
}

pub struct Game {
    puzzles: [Puzzle; LEVELS.len()],
    active: usize,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            puzzles: std::array::from_fn(|index| Puzzle::new(&LEVELS[index])),
            active: 0,
        }
    }
}

impl Game {
    pub fn restore(snapshot: Snapshot) -> Self {
        let mut game = Self::default();
        game.select(snapshot.active);
        for (index, moves) in snapshot.moves.into_iter().enumerate() {
            for direction in moves {
                if !game.puzzles[index].push(direction) {
                    game.puzzles[index] = Puzzle::new(&LEVELS[index]);
                    break;
                }
            }
        }
        game
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            active: self.active,
            moves: std::array::from_fn(|index| {
                self.puzzles[index]
                    .history
                    .iter()
                    .map(|(_, direction)| *direction)
                    .collect()
            }),
        }
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn select(&mut self, index: usize) -> bool {
        if index >= LEVELS.len() || index == self.active {
            return false;
        }
        self.active = index;
        true
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzles[self.active]
    }

    pub fn puzzle_mut(&mut self) -> &mut Puzzle {
        &mut self.puzzles[self.active]
    }

    pub fn restart(&mut self) {
        self.puzzles[self.active] = Puzzle::new(&LEVELS[self.active]);
    }

    pub fn completed(&self) -> usize {
        self.puzzles
            .iter()
            .filter(|puzzle| puzzle.complete())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLUTIONS: [&str; LEVELS.len()] = [
        "RRU",
        "LUUDRRDRUU",
        "LUURULDDDRU",
        "UURULLLLRDDRRURD",
        "LLULRUURULDDDRUU",
        "LLULDLUUURRRLDDRDLLRRRRUULU",
        "LLRRUUURULLLDDRLDLLURDRUUDDDRRUUU",
        "RURUURULLRRRDDDLUURULLRDDDDLUUUDLLDLUU",
    ];

    #[test]
    fn every_warehouse_has_a_playable_solution() {
        let mut game = Game::default();
        for (index, solution) in SOLUTIONS.into_iter().enumerate() {
            game.select(index);
            for symbol in solution.bytes() {
                let direction = match symbol {
                    b'U' => Direction::Up,
                    b'L' => Direction::Left,
                    b'D' => Direction::Down,
                    b'R' => Direction::Right,
                    _ => unreachable!(),
                };
                assert!(game.puzzle_mut().push(direction), "{}", LEVELS[index].title);
            }
            assert!(game.puzzle().complete(), "{}", LEVELS[index].title);
        }
        assert_eq!(game.completed(), LEVELS.len());
    }

    #[test]
    fn undo_and_restore_preserve_worker_crate_and_move_counts() {
        let mut game = Game::default();
        for direction in [Direction::Right, Direction::Right, Direction::Up] {
            assert!(game.puzzle_mut().push(direction));
        }
        assert!(game.puzzle().complete());
        assert!(game.puzzle_mut().undo());
        assert_eq!(game.puzzle().pushes(), 0);
        assert_eq!(game.puzzle().moves(), 2);
        assert_eq!(game.puzzle().tile(4, 3), Tile::Crate);
        game.select(4);
        let mut restored = Game::restore(game.snapshot());
        assert_eq!(restored.active(), 4);
        restored.select(0);
        assert_eq!(restored.puzzle().worker(), (4, 4));
        assert!(restored.puzzle_mut().push(Direction::Up));
        assert!(restored.puzzle().complete());
    }

    #[test]
    fn a_wall_and_two_crates_cannot_be_pushed_through() {
        let mut game = Game::default();
        game.select(1);
        assert!(game.puzzle_mut().push(Direction::Right));
        assert!(game.puzzle_mut().push(Direction::Up));
        assert!(!game.puzzle_mut().push(Direction::Left));
        assert_eq!(game.puzzle().pushes(), 0);
        assert_eq!(game.puzzle().moves(), 2);
    }
}
