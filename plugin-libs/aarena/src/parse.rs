use {Game, Intention, PointDef, PointKind, Row};

pub(crate) fn analyze_intentions<'a, I: IntoIterator<Item = &'a str>>(
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
                None => {
                    self.state = AismState::Fresh;
                    ConsumeResult::Intention(Intention::AttackLp(name.to_owned()))
                }
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

pub fn filter_commands(mut msg: &str) -> Result<Vec<&str>, ()> {
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
