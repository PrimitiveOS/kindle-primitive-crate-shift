use std::{fs, io, path::PathBuf};

use crateshift_core::{Direction, Game, Tile, COLUMNS, LEVELS, ROWS};
use primitive_app::{App, Change, HEIGHT, WIDTH};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Icon, Input, Rect, Sheet, SheetAction, TextStyle,
    BLACK, BODY, CAPTION, HAIRLINE, PAPER,
};

const CRATE: Icon = Icon {
    stroke: &["M4 4 H20 V20 H4 Z", "M4 4 L20 20 M20 4 L4 20"],
    fill: &[],
};
const RACK: Icon = Icon {
    stroke: &["M2 3 H22 M2 12 H22 M2 21 H22", "M7 3 V12 M17 12 V21"],
    fill: &[],
};

pub struct View {
    pub game: Game,
    path: Option<PathBuf>,
    confirming_reset: bool,
    notice: Option<&'static str>,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = fs::read(&path)
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .map(Game::restore)
            .unwrap_or_default();
        Self {
            game,
            path: Some(path),
            confirming_reset: false,
            notice: None,
        }
    }

    pub fn preview() -> Self {
        Self {
            game: Game::default(),
            path: None,
            confirming_reset: false,
            notice: None,
        }
    }

    fn changed(&mut self, change: Change) -> Change {
        self.notice = if self.save().is_ok() {
            None
        } else {
            Some("Could not save puzzle progress.")
        };
        change
    }

    fn step(&mut self, direction: Direction) -> Change {
        if self.game.puzzle_mut().push(direction) {
            self.changed(Change::Partial)
        } else if self.game.puzzle().complete() || self.notice == Some("That way is blocked.") {
            Change::None
        } else {
            self.notice = Some("That way is blocked.");
            Change::Partial
        }
    }

    fn swipe(&mut self, dx: f32, dy: f32, start_y: f32) -> Change {
        let width = WIDTH as i32;
        let top = board_cell(width, 0, 0).y as f32;
        let bottom = board_cell(width, 0, ROWS - 1).bottom() as f32;
        if !(top..bottom).contains(&start_y) || dx.abs().max(dy.abs()) < 40.0 {
            return Change::None;
        }

        let direction = if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        };
        self.step(direction)
    }

    fn select(&mut self, index: usize) -> Change {
        if self.game.select(index) {
            self.changed(Change::Full)
        } else {
            Change::None
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = WIDTH as i32;
        if self.confirming_reset {
            let Some(action) = reset_sheet(Rect::new(0, 0, width, HEIGHT as i32)).hit(x, y) else {
                return Change::None;
            };
            self.confirming_reset = false;
            if action == SheetAction::Confirm {
                self.game.restart();
                return self.changed(Change::Full);
            }
            return Change::Full;
        }

        let active = self.game.active();
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        if nav_button(width, 0).contains(x, y) {
            return self.select(active.saturating_sub(1));
        }
        if nav_button(width, 1).contains(x, y) && self.game.puzzle_mut().undo() {
            return self.changed(Change::Partial);
        }
        if nav_button(width, 2).contains(x, y) && self.game.puzzle().moves() > 0 {
            self.confirming_reset = true;
            return Change::Full;
        }
        if nav_button(width, 3).contains(x, y) {
            return self.select(active + 1);
        }

        let (worker_x, worker_y) = self.game.puzzle().worker();
        if let Some(direction) =
            Direction::ALL
                .into_iter()
                .enumerate()
                .find_map(|(index, direction)| {
                    let adjacent = direction
                        .destination(worker_x, worker_y)
                        .is_some_and(|(col, row)| board_cell(width, col, row).contains(x, y));
                    (adjacent || movement_button(width, index).contains(x, y)).then_some(direction)
                })
        {
            return self.step(direction);
        }
        Change::None
    }

    fn draw_board(&self, canvas: &mut Canvas, width: i32) {
        let puzzle = self.game.puzzle();
        let (worker_x, worker_y) = puzzle.worker();
        let directions = Direction::ALL.map(|direction| direction.destination(worker_x, worker_y));
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let bounds = board_cell(width, col, row);
                let tile = puzzle.tile(col, row);
                let pressed = !puzzle.complete()
                    && !self.confirming_reset
                    && directions.contains(&Some((col, row)))
                    && canvas.control_pressed(bounds, false);
                let ground = if pressed { BLACK } else { PAPER };
                let ink = if pressed { PAPER } else { BLACK };
                let length = canvas.px(24);
                canvas.fill(bounds, HAIRLINE);
                canvas.fill(bounds.inset(canvas.px(2)), ground);
                match tile {
                    Tile::Wall => {
                        let inset = bounds.inset(canvas.px(5));
                        canvas.fill(inset, BLACK);
                        canvas.icon(inset.inset(canvas.px(13)), RACK, PAPER);
                    }
                    Tile::Floor => {}
                    Tile::Worker | Tile::WorkerOnDock => {
                        if tile == Tile::WorkerOnDock {
                            canvas.circle(bounds.inset(canvas.px(14)), ink);
                            canvas.circle(bounds.inset(canvas.px(19)), ground);
                        }
                        let person = bounds.inset(length);
                        let head = person.inset(canvas.px(25));
                        canvas.circle(person, ink);
                        canvas.circle(
                            Rect::new(head.x + head.w / 4, head.y, head.w / 2, head.h / 2),
                            ground,
                        );
                        canvas.pill(
                            Rect::new(
                                head.x,
                                head.y + head.h / 2 + canvas.px(4),
                                head.w,
                                head.h / 2,
                            ),
                            0,
                            ground,
                        );
                    }
                    Tile::Dock => {
                        let dock = bounds.inset(canvas.px(40));
                        canvas.circle(dock, ink);
                        canvas.circle(dock.inset(canvas.px(7)), ground);
                        canvas.circle(dock.inset(canvas.px(23)), ink);
                    }
                    Tile::Crate | Tile::DockedCrate => {
                        let crate_rect = bounds.inset(canvas.px(12));
                        let docked = tile == Tile::DockedCrate;
                        let surface = if docked { ink } else { ground };
                        let glyph = if docked { ground } else { ink };
                        canvas.panel(crate_rect, 10, 3, surface);
                        canvas.icon(crate_rect.inset(canvas.px(12)), CRATE, glyph);
                    }
                }
            }
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let width = canvas.bounds().w;
        header(canvas, "Crate Shift", "Done");
        let index = self.game.active();
        let puzzle = self.game.puzzle();
        canvas.card(screen_rect(width, 40, 165, 1184, 109));
        canvas.text(
            screen_rect(width, 65, 182, 590, 45),
            LEVELS[index].title,
            BODY.bold(),
        );
        canvas.text(
            screen_rect(width, 65, 226, 470, 40),
            &format!("Puzzle {} of {}", index + 1, LEVELS.len()),
            CAPTION,
        );
        let (moves, pushes) = (puzzle.moves(), puzzle.pushes());
        let move_word = if moves == 1 { "move" } else { "moves" };
        let push_word = if pushes == 1 { "push" } else { "pushes" };
        canvas.text(
            screen_rect(width, 670, 182, 529, 45),
            &format!("{moves} {move_word}  ·  {pushes} {push_word}"),
            BODY.right(),
        );
        canvas.text(
            screen_rect(width, 670, 226, 529, 40),
            &format!("{} solved", self.game.completed()),
            CAPTION.right(),
        );
        let instructions = if puzzle.complete() {
            "All crates docked. Puzzle solved."
        } else {
            "Push every crate onto a marked dock."
        };
        canvas.text(
            screen_rect(width, 40, 286, 1184, 45),
            instructions,
            BODY.center(),
        );
        self.draw_board(canvas, width);

        for (index, label) in ["Up", "Left", "Down", "Right"].into_iter().enumerate() {
            draw_button(
                canvas,
                movement_button(width, index),
                label,
                !puzzle.complete(),
                false,
            );
        }
        for (index, (label, enabled, main)) in [
            ("Previous", index > 0, false),
            ("Undo", puzzle.moves() > 0, false),
            ("Restart", puzzle.moves() > 0, false),
            ("Next", index + 1 < LEVELS.len(), puzzle.complete()),
        ]
        .into_iter()
        .enumerate()
        {
            draw_button(canvas, nav_button(width, index), label, enabled, main);
        }
        let detail = self.notice.unwrap_or(if puzzle.complete() {
            "Pick another puzzle or undo your last move."
        } else {
            "Swipe or tap next to the worker to move."
        });
        canvas.text(
            screen_rect(width, 40, 1590, 1184, 70),
            detail,
            CAPTION.center(),
        );
        if self.confirming_reset {
            reset_sheet(canvas.bounds()).draw(canvas, None);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::Swipe { dx, dy, start_y } if !self.confirming_reset => {
                self.swipe(dx, dy, start_y)
            }
            Input::PageBack if self.confirming_reset => {
                self.confirming_reset = false;
                Change::Full
            }
            Input::PageBack => self.select(self.game.active().saturating_sub(1)),
            Input::PageForward if !self.confirming_reset => self.select(self.game.active() + 1),
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.path {
            primitive_app::save(path, &serde_json::to_vec(&self.game.snapshot())?)?;
        }
        Ok(())
    }
}

fn draw_button(canvas: &mut Canvas, bounds: Rect, label: &str, enabled: bool, primary: bool) {
    if enabled {
        canvas.button(bounds, label, primary, false);
    } else {
        canvas.panel(bounds, 16, 2, PAPER);
        let offset = (bounds.h - canvas.px(45)) / 2;
        canvas.text(
            Rect::new(bounds.x, bounds.y + offset, bounds.w, bounds.h - offset),
            label,
            TextStyle::new(36).bold().center().ink(115),
        );
    }
}

fn reset_sheet(bounds: Rect) -> Sheet<'static> {
    Sheet::centered(
        bounds,
        "Restart this puzzle?",
        "The crates and worker will return to their starting places.",
        "Restart",
    )
}

fn board_cell(width: i32, col: usize, row: usize) -> Rect {
    screen_rect(
        width,
        72 + col as i32 * 140,
        340 + row as i32 * 140,
        140,
        140,
    )
}

fn movement_button(width: i32, index: usize) -> Rect {
    screen_rect(width, 40 + index as i32 * 300, 1336, 284, 105)
}

fn nav_button(width: i32, index: usize) -> Rect {
    screen_rect(width, 40 + index as i32 * 300, 1464, 284, 105)
}

pub fn draw_icon(canvas: &mut Canvas) {
    canvas.begin();
    canvas.panel(Rect::new(22, 22, 212, 212), 216, 32, PAPER);
    canvas.fill(Rect::new(46, 173, 164, 20), BLACK);
    canvas.panel(Rect::new(60, 52, 136, 122), 44, 26, BLACK);
    canvas.icon(Rect::new(78, 62, 100, 100), CRATE, PAPER);
    canvas.fill(Rect::new(61, 190, 22, 16), BLACK);
    canvas.fill(Rect::new(173, 190, 22, 16), BLACK);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn swipe(dx: f32, dy: f32, start_y: f32) -> Input {
        Input::Swipe { dx, dy, start_y }
    }

    #[test]
    fn board_swipes_use_the_dominant_axis_with_vertical_ties() {
        let mut app = View::preview();
        assert_eq!(app.game.puzzle().worker(), (2, 4));
        assert_eq!(app.input(swipe(90.0, 65.0, 830.0)), Change::Partial);
        assert_eq!(app.game.puzzle().worker(), (3, 4));
        assert_eq!(app.input(swipe(-80.0, -80.0, 830.0)), Change::Partial);
        assert_eq!(app.game.puzzle().worker(), (3, 3));
        assert_eq!(app.game.puzzle().moves(), 2);
    }

    #[test]
    fn short_off_board_and_modal_swipes_leave_the_worker_in_place() {
        let mut app = View::preview();
        assert_eq!(app.input(swipe(25.0, -20.0, 830.0)), Change::None);
        assert_eq!(app.input(swipe(90.0, 65.0, 200.0)), Change::None);
        assert_eq!(app.input(swipe(90.0, 65.0, 1450.0)), Change::None);
        assert_eq!(app.input(swipe(90.0, 65.0, 830.0)), Change::Partial);
        assert_eq!(app.input(Input::Tap(790.0, 1500.0)), Change::Full);
        assert_eq!(app.input(swipe(90.0, 65.0, 830.0)), Change::None);
        assert_eq!(app.game.puzzle().worker(), (3, 4));
        assert_eq!(app.game.puzzle().moves(), 1);
    }
}
