//! A battle of wits between two superintelligences

use std::collections::HashMap;

pub struct Game {
    p1: Player,
    p2: Player,
    /// Whose turn it is
    turn: Pid,
    /// Moves left this turn. Starts at 3, except in round 1, which is summoner round.
    moves_left: u8,
    /// Number of the current round. Starts at 1.
    round: u32,
    monster_defs: HashMap<String, MonsterDef>,
    units: HashMap<String, Unit>,
}

struct MonsterDef {
    /// Attack damage of normal attack
    ad: u16,
    /// Max hitpoints
    hp: u16,
    /// Only the owner can summon it
    owner: Pid,
}

/// A unit that's out on the battlefield
struct Unit {
    /// Current attack damage
    ad: u16,
    /// Current max hp
    max_hp: u16,
    /// Current hp
    hp: u16,
    /// Current side of the battlefield
    side: Pid,
    /// Current row of the battlefield
    row: Row,
}

impl Unit {
    fn new(def: &MonsterDef, row: Row) -> Self {
        Self {
            ad: def.ad,
            max_hp: def.hp,
            hp: def.hp,
            side: def.owner,
            row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Row {
    Front,
    Back,
}

impl Game {
    pub fn new(p1name: String, p2name: String) -> Self {
        Self {
            p1: Player::new(p1name),
            p2: Player::new(p2name),
            turn: Pid::P1,
            moves_left: 1,
            round: 1,
            monster_defs: HashMap::new(),
            units: HashMap::new(),
        }
    }
    /// Returns the player whose turn it isnow
    pub fn current_player(&self) -> &Player {
        self.player_by_pid(self.turn)
    }
    pub fn current_player_pid(&self) -> Pid {
        self.turn
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
        let commands = match filter_commands(msg) {
            Ok(commands) => commands,
            Err(()) => {
                return Response {
                    lines: vec!["You should mind your square brackets, young one.".to_string()],
                    winrar: None,
                }
            }
        };
        // Ignore messages that don't contain commands
        if commands.is_empty() {
            return Response {
                lines: vec![],
                winrar: None,
            };
        }
        if pid != self.turn {
            return Response {
                lines: vec!["It's not your turn.".to_string()],
                winrar: None,
            };
        }
        let mut lines = Vec::new();
        //lines.push(format!("{:?}", commands));
        let intentions = match analyze_intentions(commands, self) {
            Ok(intentions) => intentions,
            Err(e) => {
                return Response {
                    lines: vec![e],
                    winrar: None,
                }
            }
        };
        //lines.push(format!("{:?}", intentions));
        let mut endturn = false;
        for intention in intentions {
            match intention {
                Intention::Summon { who, row } => {
                    match self.monster_defs.get(&who) {
                        Some(def) => {
                            if def.owner == self.turn {
                                use std::collections::hash_map::Entry;
                                let cpname = self.current_player().name.clone();
                                match self.units.entry(who.clone()) {
                                    Entry::Occupied(_) => {
                                        lines.push(format!("{} is already out.", who));
                                        break;
                                    }
                                    Entry::Vacant(en) => {
                                        en.insert(Unit::new(def, row));
                                        lines.push(format!(
                                            "{} summoned {} to the {:?} row.",
                                            cpname, who, row
                                        ));
                                    }
                                }
                            } else {
                                lines.push(format!(
                                    "Hey, you can't use that! It belongs to {}",
                                    self.player_by_pid(self.turn.other()).name
                                ));
                                break;
                            }
                        }
                        None => {
                            lines.push(format!(
                                "{} Doesn't exist. It's only in your imagination.",
                                who
                            ));
                            break;
                        }
                    }
                    self.moves_left -= 1;
                }
                Intention::Introduce { name, ad, hp } => {
                    lines.push("Ok.".to_owned());
                    self.monster_defs.insert(
                        name,
                        MonsterDef {
                            ad,
                            hp,
                            owner: self.turn,
                        },
                    );
                }
                Intention::Attack(attacker, target) => {
                    let mut ad = self.units[&attacker].ad;
                    if self.units[&attacker].row == Row::Back {
                        ad /= 2;
                    }
                    if self.units[&target].row == Row::Back {
                        ad /= 2;
                    }
                    lines.push(format!(
                        "{} attacks {} for {} DAMAGE.",
                        attacker, target, ad
                    ));
                    self.units.get_mut(&target).unwrap().hp =
                        self.units[&target].hp.saturating_sub(ad);
                    let hp = self.units[&target].hp;
                    if hp == 0 {
                        lines.push(format!("{} was fragged by {}", target, attacker));
                        self.units.remove(&target);
                    } else {
                        lines.push(format!("O noes, {} only has {} hp left", target, hp));
                    }
                }
                Intention::EndTurn => {
                    if self.round == 1 {
                        lines.push("YOU GOTTA SUMMON A MONSTER.".to_string());
                        break;
                    }
                    endturn = true;
                    break;
                }
            }
            if self.moves_left == 0 {
                if self.round == 1 {
                    lines.push(format!(
                        "{} finished his first summoning.",
                        self.current_player().name
                    ));
                } else {
                    lines.push(format!("{} is out of moves.", self.current_player().name));
                }
                endturn = true;
                break;
            }
        }
        if endturn {
            if self.turn == Pid::P2 {
                self.round += 1;
            }
            if self.round == 1 {
                self.moves_left = 1;
            } else {
                self.moves_left = 3;
            }
            self.turn = self.turn.other();
            lines.push(format!(
                "Now it's your turn, {}!",
                self.current_player().name
            ));
        }
        Response {
            lines,
            winrar: None,
        }
    }
    pub fn p1(&self) -> &Player {
        &self.p1
    }
    pub fn p2(&self) -> &Player {
        &self.p1
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
    game: &Game,
) -> Result<Vec<Intention>, String> {
    let mut intentions = Vec::new();
    let mut sm = Aism::new(commands.into_iter());
    loop {
        match sm.consume(game) {
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
    state: AismState<'a>,
    commands: I,
}

impl<'a, I: Iterator<Item = &'a str>> Aism<'a, I> {
    fn new(commands: I) -> Self {
        Self {
            state: AismState::Fresh,
            commands,
        }
    }
    fn consume(&mut self, gay: &Game) -> ConsumeResult {
        match self.state {
            AismState::Fresh => match self.commands.next() {
                Some(cmd) => match &cmd.to_lowercase()[..] {
                    "summon" => {
                        self.state = AismState::Summon;
                        ConsumeResult::More
                    }
                    "introduce" | "introducing" => {
                        self.state = AismState::Introduce;
                        ConsumeResult::More
                    }
                    "end" => ConsumeResult::Intention(Intention::EndTurn),
                    _ => {
                        if gay.units.contains_key(cmd) {
                            self.state = AismState::UnitRef(cmd);
                            ConsumeResult::More
                        } else {
                            ConsumeResult::Error("EXCUSE ME? WHAT?".to_string())
                        }
                    }
                },
                None => ConsumeResult::End,
            },
            AismState::Summon => match self.commands.next() {
                Some(name) => {
                    self.state = AismState::SummonName(name);
                    ConsumeResult::More
                }
                None => ConsumeResult::Error("SUMMON WHO? WHO?".to_string()),
            },
            AismState::SummonName(name) => match self.commands.next() {
                Some(row) => {
                    let row = match &row.to_lowercase()[..] {
                        "front" => Row::Front,
                        "back" => Row::Back,
                        _ => return ConsumeResult::Error("Only front or back.".to_string()),
                    };
                    self.state = AismState::Fresh;
                    ConsumeResult::Intention(Intention::Summon {
                        who: name.to_string(),
                        row,
                    })
                }
                None => ConsumeResult::Error("SUMMON TO WHICH ROW?".to_string()),
            },
            AismState::Introduce => match self.commands.next() {
                Some(name) => {
                    self.state = AismState::IntroduceName(name);
                    ConsumeResult::More
                }
                None => ConsumeResult::Error("Introduce who?".to_string()),
            },
            AismState::IntroduceName(name) => match self.commands.next() {
                Some(ad_or_hp) => {
                    let def = match parse_pointdef(ad_or_hp) {
                        Ok(def) => def,
                        Err(e) => return ConsumeResult::Error(e),
                    };
                    match def.kind {
                        PointKind::Ad => {
                            self.state = AismState::IntroduceNameAd(name, def.value);
                            ConsumeResult::More
                        }
                        PointKind::Hp => {
                            self.state = AismState::IntroduceNameHp(name, def.value);
                            ConsumeResult::More
                        }
                    }
                }
                None => ConsumeResult::Error("Need AD and HP.".to_string()),
            },
            AismState::IntroduceNameHp(name, hp) => match self.commands.next() {
                Some(ad_or_hp) => {
                    let def = match parse_pointdef(ad_or_hp) {
                        Ok(def) => def,
                        Err(e) => return ConsumeResult::Error(e),
                    };
                    match def.kind {
                        PointKind::Ad => {
                            self.state = AismState::Fresh;
                            ConsumeResult::Intention(Intention::Introduce {
                                name: name.to_owned(),
                                hp,
                                ad: def.value,
                            })
                        }
                        PointKind::Hp => {
                            ConsumeResult::Error("You gave Hp twice. Dummy.".to_string())
                        }
                    }
                }
                None => ConsumeResult::Error("Next time maybe give Ad too".to_string()),
            },
            AismState::IntroduceNameAd(name, ad) => match self.commands.next() {
                Some(ad_or_hp) => {
                    let def = match parse_pointdef(ad_or_hp) {
                        Ok(def) => def,
                        Err(e) => return ConsumeResult::Error(e),
                    };
                    match def.kind {
                        PointKind::Hp => {
                            self.state = AismState::Fresh;
                            ConsumeResult::Intention(Intention::Introduce {
                                name: name.to_owned(),
                                hp: def.value,
                                ad,
                            })
                        }
                        PointKind::Ad => {
                            ConsumeResult::Error("You gave Ad twice. Dummy.".to_string())
                        }
                    }
                }
                None => ConsumeResult::Error("Next time maybe give Hp too".to_string()),
            },
            AismState::UnitRef(name) => match self.commands.next() {
                Some(cmd) => match &cmd.to_lowercase()[..] {
                    "attack" => {
                        self.state = AismState::UnitAttack(name);
                        ConsumeResult::More
                    }
                    _ => ConsumeResult::Error("Naww. Nah. No.".to_string()),
                },
                None => ConsumeResult::Error(format!("What about {}?", name)),
            },
            AismState::UnitAttack(name) => match self.commands.next() {
                Some(unit) => {
                    if gay.units.contains_key(unit) {
                        self.state = AismState::Fresh;
                        ConsumeResult::Intention(Intention::Attack(
                            name.to_owned(),
                            unit.to_owned(),
                        ))
                    } else {
                        ConsumeResult::Error(format!(
                            "There is no {}. You must be hallucinating.",
                            unit
                        ))
                    }
                }
                None => ConsumeResult::Error("You wanna attack thin air?".to_string()),
            },
        }
    }
}

fn parse_pointdef(text: &str) -> Result<PointDef, String> {
    let mut split = text.split_whitespace();
    let value = split
        .next()
        .ok_or_else(|| "csb".to_string())?
        .trim()
        .parse::<u16>()
        .map_err(|e| e.to_string())?;
    match &split
        .next()
        .ok_or_else(|| "csb".to_string())?
        .to_lowercase()[..]
    {
        "ad" => Ok(PointDef {
            value,
            kind: PointKind::Ad,
        }),
        "hp" => Ok(PointDef {
            value,
            kind: PointKind::Hp,
        }),
        _ => Err("AD OR HP. THAT'S IT. STOP FOOLING AROUND.".to_string()),
    }
}

struct PointDef {
    value: u16,
    kind: PointKind,
}

enum PointKind {
    Ad,
    Hp,
}

enum ConsumeResult {
    Intention(Intention),
    Error(String),
    End,
    More,
}

enum AismState<'a> {
    /// Nothing is assumed yet. The default state.
    Fresh,
    /// Summon something
    Summon,
    /// Summon a monster with a name...
    SummonName(&'a str),
    /// Introduce a monster ...
    Introduce,
    /// Introduce a monster with name x, ...
    IntroduceName(&'a str),
    IntroduceNameAd(&'a str, u16),
    IntroduceNameHp(&'a str, u16),
    UnitRef(&'a str),
    UnitAttack(&'a str),
}

#[derive(Debug)]
enum Intention {
    Summon { who: String, row: Row },
    EndTurn,
    Introduce { name: String, hp: u16, ad: u16 },
    Attack(String, String),
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
    fn other(self) -> Self {
        match self {
            Pid::P1 => Pid::P2,
            Pid::P2 => Pid::P1,
        }
    }
}

pub struct Player {
    /// Name of the player. Duh.
    name: String,
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
    pub fn name(&self) -> &str {
        &self.name
    }
}
