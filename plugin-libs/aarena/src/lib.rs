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
    /// Returns the player whose turn it isnow
    pub fn current_player(&self) -> &Player {
        match self.turn {
            Pid::P1 => &self.p1,
            Pid::P2 => &self.p2,
        }
    }
    /// Interpret a message as a battle command and advance the battle.
    ///
    /// This is the MAIN SHOW.
    pub fn interpret(&mut self, msg: &str, pid: Pid) -> Response {
        if pid != self.turn {
            return Response {
                lines: vec!["It's not your turn.".to_string()],
                winrar: None,
            };
        }
        let commands = filter_commands(msg);
        Response {
            lines: vec![format!("{:?}", commands)],
            winrar: None,
        }
    }
}

fn filter_commands(mut msg: &str) -> Result<Vec<String>, ()> {
    let mut commands = Vec::new();
    loop {
        let op_br = match msg.find('[') {
            Some(pos) => pos,
            None => return Ok(commands),
        };
        msg = &msg[op_br + 1..];
        let clos_br = match msg.find(']') {
            Some(pos) => pos,
            None => return Err(()),
        };
        commands.push(msg[..clos_br].to_string());
        msg = &msg[clos_br + 1..]
    }
}

/// Interpreter state machine
enum IState {}

/// A response to whatever is running the battle about the state of the battle,
/// and messages to display.
pub struct Response {
    /// Lines of text to display
    pub lines: Vec<String>,
    /// Who won the battle, if anyone
    pub winrar: Option<Pid>,
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
