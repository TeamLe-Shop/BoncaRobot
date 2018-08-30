//! A battle of wits between two superintelligences

pub struct Game {
    pub p1: Player,
    pub p2: Player,
    turn: Pid,
}

impl Game {
    pub fn new(p1name: String, p2name: String) -> Self {
        Self {
            p1: Player::new(p1name),
            p2: Player::new(p2name),
            turn: Pid::P1,
        }
    }
}

/// Identify whether we're talking about player 1 or player 2
#[derive(PartialEq)]
pub enum Pid {
    P1,
    P2,
}

impl Pid {
    fn other(&self) -> Self {
        match *self {
            Pid::P1 => Pid::P2,
            Pid::P2 => Pid::P1,
        }
    }
}

pub struct Player {
    /// Name of the player. Duh.
    pub name: String,
    /// Level. The higher level you are, the more advanced abilities you can use.
    level: u8,
    /// Lifepoints. Game over when depleted. Starts at 500.
    lp: u16,
}

impl Player {
    fn new(name: String) -> Self {
        Self {
            name,
            lp: 500,
            level: 1,
        }
    }
}
