use fltk::{
    app,
    app::Sender,
    button::Button,
    enums::Event,
    group::VGrid,
    prelude::{ButtonExt, WidgetBase, WidgetExt},
};
use ndarray::Array2;
use rand::prelude::*;

mod message {
    #[derive(Debug, Clone)]
    pub enum MouseButton {
        Left,
        Right,
    }

    #[derive(Debug, Clone)]
    pub struct MouseMessage {
        pub button: MouseButton,
        pub location: (usize, usize),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Mine,
    Blank(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Hidden,
    Revealed,
    Uncertain,
    Marked,
}

pub struct Block {
    cell: Cell,
    state: State,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            cell: Cell::Blank(0),
            state: State::Hidden,
            // button: Button::default(),
        }
    }
}

pub struct MineMap {
    map: Array2<Block>,
    buttons: Array2<Button>,
    sender: Sender<message::MouseMessage>,
    mine: usize,
}

impl MineMap {
    pub fn new(row: usize, col: usize, mine: usize, sender: Sender<message::MouseMessage>) -> Self {
        let mut grid = VGrid::default_fill();
        grid.set_params(row as i32, col as i32, 1);

        let mut buttons = vec![];

        for _ in 0..row {
            for _ in 0..col {
                buttons.push(Button::default());
            }
        }

        let buttons = Array2::from_shape_vec((row, col), buttons).unwrap();

        grid.end();

        let mut temp = Self {
            map: Self::generate_map(row, col, mine),
            buttons,
            sender,
            mine,
        };

        temp.map_buttons();

        temp
    }

    /// handle the mouse message, update the blocks and buttons
    pub fn input(&mut self, message: message::MouseMessage) -> bool {
        let block = &mut self.map[message.location];
        let button = &mut self.buttons[message.location];

        match message.button {
            message::MouseButton::Left => match block.state {
                State::Hidden => {
                    block.state = State::Revealed;
                    button.set(true);
                    button.deactivate();

                    match block.cell {
                        Cell::Mine => {
                            self.show_all();
                            return false;
                        }
                        Cell::Blank(0) => {
                            for r in [
                                message.location.0.wrapping_sub(1),
                                message.location.0,
                                message.location.0 + 1,
                            ] {
                                for c in [
                                    message.location.1.wrapping_sub(1),
                                    message.location.1,
                                    message.location.1 + 1,
                                ] {
                                    if (r, c) != (message.location.0, message.location.1) {
                                        if let Some(b) = self.map.get_mut((r, c)) {
                                            if let (Cell::Blank(_), State::Hidden) =
                                                (b.cell, b.state)
                                            {
                                                self.input(message::MouseMessage {
                                                    button: message::MouseButton::Left,
                                                    location: (r, c),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Cell::Blank(_) => {}
                    }
                }
                _ => {}
            },
            message::MouseButton::Right => match block.state {
                State::Hidden => {
                    block.state = State::Marked;
                }
                State::Marked => {
                    block.state = State::Uncertain;
                }
                State::Uncertain => {
                    block.state = State::Hidden;
                }
                State::Revealed => {}
            },
        }

        self.flush_display();

        true
    }

    /// check the winning condition, return true if win, otherwise false
    pub fn check_win(&mut self) -> bool {
        for block in self.map.iter() {
            let cell = &block.cell;
            if *cell != Cell::Mine {
                if block.state != State::Revealed {
                    return false;
                }
            }
        }
        self.buttons.iter_mut().for_each(|b| b.deactivate());
        true
    }

    /// restart the same game
    pub fn restart_same(&mut self) {
        self.map.indexed_iter_mut().for_each(|(location, block)| {
            let button = &mut self.buttons[location];
            button.set_label("");
            button.set(false);
            button.activate();
            block.state = State::Hidden;
        });
    }

    /// restart a new game
    pub fn restart(&mut self) {
        let (r, c) = (self.map.shape()[0], self.map.shape()[1]);
        self.map = Self::generate_map(r, c, self.mine);

        self.map_buttons();
        self.buttons.iter_mut().for_each(|button| {
            button.set_label("");
            button.set(false);
            button.activate();
        });

        self.flush_display();
    }

    /// set buttons to show all blocks, used when lose
    fn show_all(&mut self) {
        self.map
            .iter_mut()
            .for_each(|block| block.state = State::Revealed);
        self.buttons.iter_mut().for_each(|b| b.set(true));
        self.flush_display();
    }

    /// set buttons to show correspond block
    fn flush_display(&mut self) {
        for (location, block) in self.map.indexed_iter_mut() {
            let button = &mut self.buttons[location];
            match block.state {
                State::Hidden => button.set_label(""),
                State::Revealed => match block.cell {
                    Cell::Mine => button.set_label("💣"),
                    Cell::Blank(0) => {}
                    Cell::Blank(n) => button.set_label(format!("{}", n).as_str()),
                },
                State::Uncertain => button.set_label("❓"),
                State::Marked => button.set_label("🚩"),
            }
        }
    }

    /// add callback to buttons
    fn map_buttons(&mut self) {
        self.map.indexed_iter_mut().for_each(|(location, _)| {
            let sender = self.sender.clone();
            self.buttons[location].handle(move |_, e| match e {
                Event::Push => match app::event_mouse_button() {
                    app::MouseButton::Left => {
                        sender.send(message::MouseMessage {
                            button: message::MouseButton::Left,
                            location,
                        });
                        true
                    }
                    app::MouseButton::Right => {
                        sender.send(message::MouseMessage {
                            button: message::MouseButton::Right,
                            location,
                        });
                        true
                    }
                    _ => false,
                },
                _ => false,
            });
        });
    }

    fn generate_map(
        row: usize,
        col: usize,
        mine: usize,
    ) -> ndarray::ArrayBase<ndarray::OwnedRepr<Block>, ndarray::Dim<[usize; 2]>> {
        let mut v: Vec<bool> = vec![false; row * col];
        for i in &mut v[0..mine] {
            *i = true;
        }

        /* shuffle */
        let mut rng = rand::thread_rng();
        v.shuffle(&mut rng);

        let v = v
            .into_iter()
            .map(|b| match b {
                false => Block::default(),
                true => {
                    let mut block = Block::default();
                    block.cell = Cell::Mine;
                    block
                }
            })
            .collect::<Vec<_>>();

        let mut arr2 = Array2::from_shape_vec((row, col), v).unwrap();

        /* update blank number */
        for r in 0..row {
            for c in 0..col {
                if arr2[(r, c)].cell == Cell::Mine {
                    for i in [r.wrapping_sub(1), r, r.wrapping_add(1)] {
                        for j in [c.wrapping_sub(1), c, c.wrapping_add(1)] {
                            if let Some(block) = arr2.get_mut((i, j)) {
                                if let Cell::Blank(n) = block.cell {
                                    block.cell = Cell::Blank(n + 1);
                                }
                            }
                        }
                    }
                }
            }
        }
        arr2
    }
}
