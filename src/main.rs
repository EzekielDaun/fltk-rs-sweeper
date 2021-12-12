// #![windows_subsystem = "windows"]
use fltk::{app, button::Button, dialog::HelpDialog, group, prelude::*, window::Window};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

mod mine_map;

const BUT_SIZE: i32 = 36;
const ROW: i32 = 8;
const COL: i32 = 10;
const TOP: i32 = 36;

const EMOJI_NORMAL: &str = "😀";
const EMOJI_WIN: &str = "😄";
const EMOJI_LOSE: &str = "☹";

static TIME_COUNT: AtomicU32 = AtomicU32::new(0);
static RESTART_BUTTON: AtomicBool = AtomicBool::new(false);
static WIN: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicBool = AtomicBool::new(false);

fn main() {
    let app = app::App::default();
    let mut wind = Window::default()
        .with_size(BUT_SIZE * COL, BUT_SIZE * ROW + TOP)
        .center_screen()
        .with_label("fltk-sweeper");

    let (mouse_sender, mouse_receiver) = app::channel();

    let mut col = group::Column::default_fill();
    col.set_margin(0);
    col.set_pad(0);

    let mut group1 = group::Group::default().with_size(BUT_SIZE * COL, TOP);
    let mut button = Button::default()
        .with_size(36, 36)
        .with_label(EMOJI_NORMAL)
        .center_of_parent();
    button.handle(move |_, e| match e {
        fltk::enums::Event::Push => {
            RESTART_BUTTON.fetch_or(true, Ordering::SeqCst);
            true
        }
        _ => false,
    });
    let mut time = Button::default().with_size(3 * BUT_SIZE, BUT_SIZE).left_of(
        &button,
        (group1.width() - button.width()) / 2 - 3 * BUT_SIZE,
    );
    time.deactivate();
    group1.end();

    let group2 = group::Group::default().with_size(BUT_SIZE * COL, BUT_SIZE * ROW);
    let mut map = mine_map::MineMap::new(ROW as usize, COL as usize, 8, mouse_sender.clone());
    group2.end();

    col.set_size(&mut group1, TOP);

    col.end();

    wind.end();
    wind.show();

    app::add_timeout(0.01, callback_per_second);

    while app.wait() {
        if let Some(msg) = mouse_receiver.recv() {
            if !STARTED.load(Ordering::SeqCst) {
                STARTED.store(true, Ordering::SeqCst);
            }
            if !map.input(msg) {
                button.set_label(EMOJI_LOSE);
                let choice = fltk::dialog::choice_default(
                    format!("You lose").as_str(),
                    "Retry",
                    "New Game",
                    "Quit",
                );
                if choice == 0 {
                    STARTED.store(false, Ordering::SeqCst);
                    map.restart_same();
                } else if choice == 1 {
                    STARTED.store(false, Ordering::SeqCst);
                    map.restart();
                } else {
                    app.quit();
                }
            };
            if map.check_win() {
                button.set_label(EMOJI_WIN);
                WIN.store(true, Ordering::SeqCst);
                STARTED.store(false, Ordering::SeqCst);

                let choice = fltk::dialog::choice_default(
                    format!("You Win").as_str(),
                    "Retry",
                    "New Game",
                    "Quit",
                );
                if choice == 0 {
                    STARTED.store(false, Ordering::SeqCst);
                    map.restart_same();
                } else if choice == 1 {
                    STARTED.store(false, Ordering::SeqCst);
                    map.restart();
                } else {
                    app.quit();
                }
            }
        }
        if RESTART_BUTTON.load(Ordering::SeqCst) {
            RESTART_BUTTON.store(false, Ordering::SeqCst);
            WIN.store(false, Ordering::SeqCst);
            STARTED.store(false, Ordering::SeqCst);
            map.restart();
        }
        if !WIN.load(Ordering::SeqCst) {
            time.set_label(
                format!(
                    "time:{:.2}",
                    TIME_COUNT.load(Ordering::SeqCst) as f32 / 100.0
                )
                .as_str(),
            );
        }
    }
}

fn callback_per_second() {
    app::repeat_timeout(0.01, callback_per_second);
    if STARTED.load(Ordering::SeqCst) {
        TIME_COUNT.fetch_add(1, Ordering::SeqCst);
    } else {
        TIME_COUNT.store(0, Ordering::SeqCst);
    }
}
