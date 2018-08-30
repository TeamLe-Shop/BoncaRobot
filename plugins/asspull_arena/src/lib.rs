#[macro_use]
extern crate plugin_api;

extern crate aarena;

use aarena::Game;
use plugin_api::prelude::*;

enum AssPullState {
    /// Inactive state. Nothing in progress.
    Inactive,
    /// An in-progress challenge request from a player towards another player
    ChallengeReq {
        challenger: String,
        challengee: String,
    },
    /// Game initiated
    Game(Game),
}

struct AssPull {
    /// Current state
    state: AssPullState,
}

impl AssPull {
    fn challenge(this: &mut Plugin, arg: &str, ctx: Context) {
        let this: &mut Self = this.downcast_mut().unwrap();
        match this.state {
            AssPullState::Inactive => {
                let p1nick = ctx.sender.nickname();
                ctx.send_channel(&format!(
                    "{}: {} challenges you to a duel. Use '.accept' to accept the challenge.",
                    arg, p1nick,
                ));
                this.state = AssPullState::ChallengeReq {
                    challenger: p1nick.to_string(),
                    challengee: arg.to_string(),
                };
            }
            AssPullState::ChallengeReq {
                ref challenger,
                ref challengee,
            } => ctx.send_channel(&format!(
                "{}: Sorry, a challenge request is already in progress ({} vs {}).",
                ctx.sender.nickname(),
                challenger,
                challengee,
            )),
            AssPullState::Game(ref g) => {
                ctx.send_channel(&format!(
                    "{}: Sorry, a game is already in progress between {} and {}.",
                    ctx.sender.nickname(),
                    g.p1.name,
                    g.p2.name,
                ));
            }
        }
    }
    fn accept_challenge(this: &mut Plugin, _arg: &str, ctx: Context) {
        let this: &mut Self = this.downcast_mut().unwrap();
        let mut game = None;
        match this.state {
            AssPullState::Inactive => {
                ctx.send_channel(&format!("{}: Uhh....", ctx.sender.nickname()))
            }
            AssPullState::ChallengeReq {
                ref challenger,
                ref challengee,
            } => {
                let nick = ctx.sender.nickname();
                if *nick == *challengee {
                    ctx.send_channel(&format!(
                        "{} VS {} - LET THE BATTLE BEGIN",
                        challenger, challengee
                    ));
                    game = Some(Game::new(challenger.clone(), challengee.clone()));
                } else {
                    ctx.send_channel(&format!("{}: Foreveralone", ctx.sender.nickname()));
                }
            }
            AssPullState::Game(ref g) => {
                ctx.send_channel(&format!(
                    "{}: Cool story bro. Keep watching {} vs {}",
                    ctx.sender.nickname(),
                    g.p1.name,
                    g.p2.name
                ));
            }
        }
        if let Some(g) = game {
            this.state = AssPullState::Game(g);
        }
    }
    fn chicken(this: &mut Plugin, _arg: &str, ctx: Context) {
        let this: &mut Self = this.downcast_mut().unwrap();
        let nick = ctx.sender.nickname();
        let mut cancel = false;
        match this.state {
            AssPullState::Inactive => {
                ctx.send_channel("TO GET TO THE OTHER SIDE");
            }
            AssPullState::ChallengeReq {
                ref challenger,
                ref challengee,
            } => {
                if *nick == *challenger {
                    ctx.send_channel(&format!("Hah, {} is a big chicken. He challenged {}, then immediately chickened out! What a coward! Let's all laugh at him! HAHAHAHAHAHAHAHA!", challenger, challengee));
                    cancel = true;
                } else if *nick == *challengee {
                    ctx.send_channel(&format!(
                        "{} is a chicken. He cowers in fear of {}",
                        challengee, challenger
                    ));
                    cancel = true;
                } else {
                    ctx.send_channel(&format!("{} got to the other side.", nick));
                }
            }
            AssPullState::Game(ref g) => {
                if *nick == *g.p1.name {
                    ctx.send_channel(&format!(
                        "{} IS A BIG CHICKEN! {} WINS!",
                        g.p1.name, g.p2.name
                    ));
                    cancel = true;
                } else if *nick == *g.p2.name {
                    ctx.send_channel(&format!(
                        "{} IS A BIG CHICKEN! {} WINS!",
                        g.p2.name, g.p1.name
                    ));
                    cancel = true
                } else {
                    ctx.send_channel(&format!("{} got to the other side.", nick));
                }
            }
        }
        if cancel {
            this.state = AssPullState::Inactive;
        }
    }
}

impl Plugin for AssPull {
    fn new() -> Self {
        Self {
            state: AssPullState::Inactive,
        }
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("challenge", "Challenge someone to a duel", Self::challenge);
        meta.command(
            "accept",
            "Accept the challenge to a duel",
            Self::accept_challenge,
        );
        meta.command("chicken", "Chicken out", Self::chicken);
    }
}

plugin_export!(AssPull);
