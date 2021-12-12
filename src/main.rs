// #![windows_subsystem = "windows"]
use fltk::{app, button::Button, group, prelude::*, window::Window};
use std::{
    cmp,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};
use structopt::StructOpt;

mod mine_map;

const BUT_SIZE: i32 = 36;
const TOP: i32 = 36;

const EMOJI_NORMAL: &str = "😀";
const EMOJI_WIN: &str = "😄";
const EMOJI_LOSE: &str = "☹";

static TIME_COUNT: AtomicU32 = AtomicU32::new(0);
static RESTART_BUTTON: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicBool = AtomicBool::new(false);

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// number of rows
    #[structopt(short, long, default_value = "8")]
    row: usize,

    /// number of columns
    #[structopt(short, long, default_value = "10")]
    col: usize,

    /// number of mines
    #[structopt(short, long, default_value = "6")]
    mine: usize,
}

fn main() {
    let opt = Opt::from_args();
    let (row_num, col_num, mine_num) = (opt.row, cmp::max(opt.col, 5), opt.mine);

    let app = app::App::default();
    let mut wind = Window::default()
        .with_size(BUT_SIZE * col_num as i32, BUT_SIZE * row_num as i32 + TOP)
        .center_screen()
        .with_label("fltk-sweeper");

    let (mouse_sender, mouse_receiver) = app::channel();

    let mut col = group::Column::default_fill();
    col.set_margin(0);
    col.set_pad(0);

    let mut group1 = group::Group::default().with_size(BUT_SIZE * col_num as i32, TOP);
    let mut restart_button = Button::default()
        .with_size(36, TOP)
        .with_label(EMOJI_NORMAL)
        .center_of_parent();
    restart_button.handle(move |_, e| match e {
        fltk::enums::Event::Push => {
            RESTART_BUTTON.fetch_or(true, Ordering::SeqCst);
            true
        }
        _ => false,
    });

    let mut time = Button::default().with_size(2 * BUT_SIZE, BUT_SIZE).left_of(
        &restart_button,
        (group1.width() - restart_button.width()) / 2 - 2 * BUT_SIZE,
    );
    time.set(true);
    time.deactivate();
    group1.end();

    let group2 =
        group::Group::default().with_size(BUT_SIZE * col_num as i32, BUT_SIZE * row_num as i32);
    let mut map = mine_map::MineMap::new(
        row_num as usize,
        col_num as usize,
        mine_num,
        mouse_sender.clone(),
    );
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
                restart_button.set_label(EMOJI_LOSE);
                let choice = fltk::dialog::choice_default(
                    format!("You lose").as_str(),
                    "Retry",
                    "New Game",
                    "Quit",
                );
                if choice == 0 {
                    STARTED.store(false, Ordering::SeqCst);
                    restart_button.set_label(EMOJI_NORMAL);
                    map.restart_same();
                } else if choice == 1 {
                    STARTED.store(false, Ordering::SeqCst);
                    restart_button.set_label(EMOJI_NORMAL);
                    map.restart();
                } else {
                    app.quit();
                }
            };
            if map.check_win() {
                restart_button.set_label(EMOJI_WIN);
                STARTED.store(false, Ordering::SeqCst);
                let choice = fltk::dialog::choice_default(
                    format!(
                        "You Win\nTime:{:.2}",
                        TIME_COUNT.load(Ordering::SeqCst) as f32 / 100.0
                    )
                    .as_str(),
                    "Retry",
                    "New Game",
                    "Quit",
                );
                if choice == 0 {
                    STARTED.store(false, Ordering::SeqCst);
                    restart_button.set_label(EMOJI_NORMAL);
                    map.restart_same();
                } else if choice == 1 {
                    STARTED.store(false, Ordering::SeqCst);
                    restart_button.set_label(EMOJI_NORMAL);
                    map.restart();
                } else {
                    app.quit();
                }
            }
        }
        if RESTART_BUTTON.load(Ordering::SeqCst) {
            restart_button.set_label(EMOJI_NORMAL);
            RESTART_BUTTON.store(false, Ordering::SeqCst);
            STARTED.store(false, Ordering::SeqCst);
            map.restart();
        }

        time.set_label(format!("{:.2}", TIME_COUNT.load(Ordering::SeqCst) as f32 / 100.0).as_str());
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
