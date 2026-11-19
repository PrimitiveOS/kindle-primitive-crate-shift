use std::{
    fs,
    path::{Path, PathBuf},
};

use crateshift_render::{draw_icon, View};
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{screen_rect, Canvas, Input, BLACK, PAPER};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let paths = font_paths(&fonts);
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &paths)?;
    let mut icon = Canvas::new(256, 256, &paths)?;
    draw_icon(&mut icon);
    write_png(&output.join("icon.png"), 256, 256, &icon.finish(false))?;

    let mut app = View::preview();
    write_png(
        &output.join("crateshift.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, false),
    )?;

    let mut dock_view = View::preview();
    for x in [210.0, 210.0, 1050.0, 1050.0] {
        assert_eq!(dock_view.input(Input::Tap(x, 1374.0)), Change::Partial);
    }
    let dock = screen_rect(WIDTH as i32, 632, 620, 140, 140);
    let column = (dock.x + dock.w / 2) as usize;
    let rim = (dock.y + canvas.px(16)) as usize * WIDTH as usize + column;
    let gap = (dock.y + canvas.px(21)) as usize * WIDTH as usize + column;
    let occupied = frame(&dock_view, &mut canvas, false);
    assert_eq!(occupied[rim], BLACK);
    assert_eq!(occupied[gap], PAPER);

    assert_eq!(
        app.input(Input::Swipe {
            dx: 170.0,
            dy: 30.0,
            start_y: 954.0
        }),
        Change::Partial
    );
    assert_eq!(app.input(Input::Tap(1050.0, 1374.0)), Change::Partial);
    assert_eq!(app.input(Input::Tap(210.0, 1374.0)), Change::Partial);
    assert!(app.game.puzzle().complete());
    assert_eq!(app.input(Input::Tap(460.0, 1500.0)), Change::Partial);
    assert!(!app.game.puzzle().complete());
    assert_eq!(app.input(Input::PageForward), Change::Full);
    assert_eq!(app.game.active(), 1);
    assert_eq!(app.input(Input::PageBack), Change::Full);
    assert_eq!(app.game.puzzle().moves(), 2);

    app.game.select(7);
    assert_eq!(app.input(Input::Tap(1050.0, 1374.0)), Change::Partial);
    assert_eq!(app.input(Input::Tap(210.0, 1374.0)), Change::Partial);
    write_png(
        &output.join("crateshift-playing.png"),
        WIDTH,
        HEIGHT,
        &frame(&app, &mut canvas, true),
    )?;
    assert_eq!(app.input(Input::Tap(790.0, 1500.0)), Change::Full);
    assert_eq!(app.input(Input::Tap(846.0, 1000.0)), Change::Full);
    assert_eq!(app.game.puzzle().moves(), 0);
    assert_eq!(app.input(Input::Tap(1120.0, 82.0)), Change::Exit);
    Ok(())
}
