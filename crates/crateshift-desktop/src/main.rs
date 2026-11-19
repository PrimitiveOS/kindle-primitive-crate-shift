use std::{
    fs,
    path::{Path, PathBuf},
};

use crateshift_render::{draw_icon, View};
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};

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

    for _ in 0..2 {
        assert_eq!(app.input(Input::Tap(1050.0, 1374.0)), Change::Partial);
    }
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
