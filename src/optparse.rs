#[test]
fn test_parse() {
    let opt_defs = [
        OptDef {
            short: 'f',
            long: "flag",
            help: "A simple flag",
            takes_args: false,
        },
        OptDef {
            short: 'o',
            long: "opt",
            help: "Takes one arg",
            takes_args: true,
        },
        OptDef {
            short: 'm',
            long: "multi",
            help: "Takes multiple args",
            takes_args: true,
        },
    ];
    let args = parse(
        "-f --flag free1 -o arg -m [multi, args] --opt arg free2 --multi [multi, args] free3",
        &opt_defs,
    );
    let expected = ParsedOpts {
        opts: vec![
            Opt {
                name: "flag",
                args: vec![],
            },
            Opt {
                name: "flag",
                args: vec![],
            },
            Opt {
                name: "opt",
                args: vec!["arg".to_owned()],
            },
            Opt {
                name: "multi",
                args: vec!["multi".to_owned(), "args".to_owned()],
            },
            Opt {
                name: "opt",
                args: vec!["arg".to_owned()],
            },
            Opt {
                name: "multi",
                args: vec!["multi".to_owned(), "args".to_owned()],
            },
        ],
        free: vec!["free1".to_owned(), "free2".to_owned(), "free3".to_owned()],
    };
    assert_eq!(args.unwrap(), expected);
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
}

impl ParseError {
    fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
enum ParseErrorKind {
    UnexpectedChar(char),
    UnexpectedEnd,
    UnrecognizedShort(char),
    UnrecognizedLong(String),
}

pub fn parse(mut cmdline: &str, opt_defs: &[OptDef]) -> Result<ParsedOpts, ParseError> {
    let mut parsed_opts = ParsedOpts::default();
    loop {
        match next_nonwhite(cmdline.char_indices()) {
            Some((idx, '-')) => {
                cmdline = (parse_opt(&cmdline[idx + 1..], opt_defs, &mut parsed_opts))?;
            }
            Some((idx, _)) => {
                cmdline = (parse_free(&cmdline[idx..], &mut parsed_opts))?;
            }
            None => panic!("I didn't expect this"),
        }
        if cmdline.is_empty() {
            return Ok(parsed_opts);
        }
    }
}

fn parse_opt<'a>(
    cmdline: &'a str,
    opt_defs: &[OptDef],
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    if let Some('-') = cmdline.chars().next() {
        parse_long(&cmdline[1..], opt_defs, parsed_opts)
    } else {
        parse_short(cmdline, opt_defs, parsed_opts)
    }
}

fn parse_free<'a>(cmdline: &'a str, parsed_opts: &mut ParsedOpts) -> Result<&'a str, ParseError> {
    match cmdline.find('-') {
        Some(pos) => {
            parsed_opts.free.push(cmdline[..pos].trim().to_owned());
            Ok(&cmdline[pos..])
        }
        None => {
            parsed_opts.free.push(cmdline.to_owned());
            Ok("")
        }
    }
}

fn parse_short<'a>(
    cmdline: &'a str,
    opt_defs: &[OptDef],
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    match cmdline.chars().next() {
        None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd)),
        Some('-') => Err(ParseError::new(ParseErrorKind::UnexpectedChar('-'))),
        Some(c) => match lookup_by_short(opt_defs, c) {
            Some(def) => {
                parsed_opts.opts.push(Opt {
                    name: def.long,
                    args: vec![],
                });
                if def.takes_args {
                    parse_1_or_more_args(&cmdline[1..], parsed_opts)
                } else {
                    Ok(&cmdline[1..])
                }
            }
            None => Err(ParseError::new(ParseErrorKind::UnrecognizedShort(c))),
        },
    }
}

fn parse_long<'a>(
    cmdline: &'a str,
    opt_defs: &[OptDef],
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    let (long, end) = match cmdline.find(' ') {
        Some(pos) => (&cmdline[..pos], pos),
        None => (cmdline, cmdline.len()),
    };
    match lookup_by_long(opt_defs, long) {
        Some(def) => {
            parsed_opts.opts.push(Opt {
                name: def.long,
                args: vec![],
            });
            if def.takes_args {
                (parse_1_or_more_args(&cmdline[end + 1..], parsed_opts))
            } else {
                Ok(&cmdline[std::cmp::min(end + 1, cmdline.len())..])
            }
        }
        None => Err(ParseError::new(ParseErrorKind::UnrecognizedLong(
            long.to_owned(),
        ))),
    }
}

fn parse_1_or_more_args<'a>(
    cmdline: &'a str,
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    match next_nonwhite(cmdline.char_indices()) {
        None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd)),
        Some((idx, '[')) => (parse_arg_array(&cmdline[idx + 1..], parsed_opts)),
        Some((idx, _)) => (parse_single_arg(&cmdline[idx..], parsed_opts)),
    }
}

fn next_nonwhite(mut iter: impl Iterator<Item = (usize, char)>) -> Option<(usize, char)> {
    loop {
        let (idx, ch) = iter.next()?;
        if ch != ' ' {
            return Some((idx, ch));
        }
    }
}

fn parse_arg_array<'a>(
    cmdline: &'a str,
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    match cmdline.find(']') {
        Some(end) => {
            let args: Vec<String> = cmdline[..end]
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect();
            parsed_opts.opts.last_mut().unwrap().args = args;
            (Ok(&cmdline[end + 1..]))
        }
        None => panic!("parse error unterminated ["),
    }
}

fn parse_single_arg<'a>(
    cmdline: &'a str,
    parsed_opts: &mut ParsedOpts,
) -> Result<&'a str, ParseError> {
    match cmdline.find(|c| c != ' ') {
        Some(begin) => match cmdline[begin..].find(' ') {
            Some(end) => {
                let arg = &cmdline[begin..begin + end];
                parsed_opts
                    .opts
                    .last_mut()
                    .unwrap()
                    .args
                    .push(arg.to_owned());
                (Ok(&cmdline[begin + end..]))
            }
            None => {
                let arg = &cmdline[begin..];
                parsed_opts
                    .opts
                    .last_mut()
                    .unwrap()
                    .args
                    .push(arg.to_owned());
                Ok("")
            }
        },
        None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd)),
    }
}

fn lookup_by_short(opt_defs: &[OptDef], short: char) -> Option<&OptDef> {
    for opt in opt_defs {
        if opt.short == short {
            return Some(opt);
        }
    }
    None
}

fn lookup_by_long<'a>(opt_defs: &'a [OptDef], long: &str) -> Option<&'a OptDef> {
    for opt in opt_defs {
        if opt.long == long {
            return Some(opt);
        }
    }
    None
}

pub struct OptDef {
    pub short: char,
    pub long: &'static str,
    pub help: &'static str,
    pub takes_args: bool,
}

#[derive(Debug, PartialEq)]
pub struct Opt {
    name: &'static str,
    args: Vec<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct ParsedOpts {
    pub opts: Vec<Opt>,
    pub free: Vec<String>,
}

impl ParsedOpts {
    pub fn get_or_empty(&self, name: &str) -> &[String] {
        for opt in &self.opts {
            if opt.name == name {
                return &opt.args[..];
            }
        }
        &[]
    }
    pub fn given(&self, name: &str) -> bool {
        for opt in &self.opts {
            if opt.name == name {
                return true;
            }
        }
        false
    }
}
