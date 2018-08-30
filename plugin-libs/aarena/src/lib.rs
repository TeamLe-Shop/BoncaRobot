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
        self.player_by_pid(self.turn)
    }
    pub fn player_by_pid(&self, pid: Pid) -> &Player {
        match pid {
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
        let commands = match filter_commands(msg) {
            Ok(commands) => commands,
            Err(()) => {
                return Response {
                    lines: vec!["You should mind your square brackets, young one.".to_string()],
                    winrar: None,
                }
            }
        };
        let mut lines = Vec::new();
        //lines.push(format!("{:?}", commands));
        let intentions = match analyze_intentions(commands) {
            Ok(intentions) => intentions,
            Err(e) => {
                return Response {
                    lines: vec![e],
                    winrar: None,
                }
            }
        };
        //lines.push(format!("{:?}", intentions));
        for intention in intentions {
            match intention {
                Intention::Summon { who } => {
                    lines.push(format!("{} summoned {}.", self.current_player().name, who))
                }
                Intention::EndTurn => {
                    self.turn = self.turn.other();
                    lines.push(format!(
                        "Now it's your turn, {}!",
                        self.current_player().name
                    ));
                }
            }
        }
        Response {
            lines,
            winrar: None,
        }
    }
}

fn filter_commands(mut msg: &str) -> Result<Vec<&str>, ()> {
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
        commands.push(&msg[..clos_br]);
        msg = &msg[clos_br + 1..]
    }
}

fn analyze_intentions<'a, I: IntoIterator<Item = &'a str>>(
    commands: I,
) -> Result<Vec<Intention>, String> {
    let mut intentions = Vec::new();
    let mut sm = Aism::new(commands.into_iter());
    loop {
        match sm.consume() {
            ConsumeResult::Intention(intention) => {
                intentions.push(intention);
            }
            ConsumeResult::Error(e) => return Err(e),
            ConsumeResult::End => break,
            ConsumeResult::More => {}
        }
    }
    Ok(intentions)
}

/// Analyze Intentions State Machine
struct Aism<'a, I: Iterator<Item = &'a str>> {
    state: AismState,
    commands: I,
}

impl<'a, I: Iterator<Item = &'a str>> Aism<'a, I> {
    fn new(commands: I) -> Self {
        Self {
            state: AismState::Fresh,
            commands,
        }
    }
    fn consume(&mut self) -> ConsumeResult {
        match self.state {
            AismState::Fresh => match self.commands.next() {
                Some(cmd) => match &cmd.to_lowercase()[..] {
                    "summon" => {
                        self.state = AismState::Summon;
                        ConsumeResult::More
                    }
                    "end" => ConsumeResult::Intention(Intention::EndTurn),
                    _ => ConsumeResult::Error("EXCUSE ME? WHAT?".to_string()),
                },
                None => ConsumeResult::End,
            },
            AismState::Summon => match self.commands.next() {
                Some(name) => {
                    self.state = AismState::Fresh;
                    ConsumeResult::Intention(Intention::Summon {
                        who: name.to_string(),
                    })
                }
                None => ConsumeResult::Error("SUMMON WHO? WHO?".to_string()),
            },
        }
    }
}

enum ConsumeResult {
    Intention(Intention),
    Error(String),
    End,
    More,
}

enum AismState {
    /// Nothing is assumed yet. The default state.
    Fresh,
    /// Summon something
    Summon,
}

#[derive(Debug)]
enum Intention {
    Summon { who: String },
    EndTurn,
}

/// A response to whatever is running the battle about the state of the battle,
/// and messages to display.
pub struct Response {
    /// Lines of text to display
    pub lines: Vec<String>,
    /// Who won the battle, if anyone
    pub winrar: Option<Pid>,
}

/// Identify whether we're talking about player 1 or player 2
#[derive(PartialEq, Clone, Copy)]
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
