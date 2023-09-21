use funk::FunkDb;
use typed_builder::TypedBuilder;
use std::{ffi::OsString, sync::RwLockWriteGuard};
use anyhow::{Result, Error, bail, anyhow, self as ah};

fn main() -> anyhow::Result<()> {
    let argv: Vec<OsString> = std::env::args_os().into_iter().skip(1_usize).collect();
    let op = parse_cli(argv);
    dispatch(op)?;
    Ok(())
}

fn parse_cli(argv: Vec<OsString>) -> Operation {
    if argv.len() == 0 { return Operation::default(); }
    if argv.len() == 1 {
        let mut it = argv[0].to_ascii_lowercase().into_string().expect("valid unicode");
        match it.as_str() {
            "create" => {
                return op_help_create();
            }
            "open" => {
                return op_help_open();
            }
            "repl" => {
                return op_repl();
            }
            _ => {
                return Operation::default();
            }
        }
    }
    
    let mut op = {
        let it = argv[0].to_ascii_lowercase().into_string().expect("valid unicode");     
        match it.as_str() {
            "create" => {
                Operation::builder().mode(Mode::Create)
            }
            "repl" => {
                Operation::builder().mode(Mode::EmptyRepl)
            }
            "open" => {
                Operation::builder().mode(Mode::Open)
            }
            "help" => {
                let help_term = argv[1].to_ascii_lowercase().into_string().expect("...");
                match help_term.as_str() {
                    "create" => { return op_help_create(); }
                    "repl" => { return op_help_repl(); }
                    "open" => { return op_help_open(); }
                    "help" => { return op_help_help(); }
                    _ => { return Operation::default(); }
                }
            }
            _ => {
                return Operation::default();
            }
        }
    };

    // TODO: Extract keyword arguments (--thing="that")
    // For now, we'll just crash if there's a '--' in the concatenated argv vector
    const DANGER: &str = "--";
    for word in argv.clone().into_iter() {
        if word.to_ascii_lowercase().into_string().unwrap().contains(DANGER) {
            eprintln!("We do not yet handle keyword arguments.");
            return Operation::default();
        }
    }

    let op_args: Args = {
        let mut inner: Vec<String> = 
            argv[1..]
                .to_owned()
                .into_iter()
                .map(|x| x.to_str().unwrap().to_string()).collect();
        Args(inner)        
    };    

    op.args(op_args).build()    
}

fn op_repl() -> Operation {
    Operation::builder().mode(Mode::EmptyRepl).build()
}

fn op_help_create() -> Operation {
    let op_code = isize::from(Mode::Create);
    Operation::builder()
        .mode(Mode::Create)
        .build()
}

fn op_help_open() -> Operation {
    let op_code = isize::from(Mode::Open);
    Operation::builder().mode(Mode::Help(HelpKind::ModeHelp(op_code))).build()
}

fn op_help_help() -> Operation {
    Operation::builder()
        .mode(Mode::HelpHelp)
        .build()    
}

fn op_help_repl() -> Operation {
    let op_code = isize::from(Mode::EmptyRepl);
    Operation::builder()
        .mode(Mode::Help(HelpKind::ModeHelp(op_code))).build()
}

#[non_exhaustive]
#[derive(TypedBuilder, Debug, Default, Eq, PartialEq, Clone)]
pub(crate) struct Operation {
    #[builder(default)]
    mode: Mode,
    #[builder(default, setter(strip_option))]
    args: Option<Args>,
    #[builder(default, setter(strip_option))]
    kwargs: Option<Kwargs>,
}




#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct Args(pub Vec<(/* arg = */ String)>);

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct Kwargs(pub Vec<(/* key = */ String, /* value = */ String)>);


#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum Mode {
    HelpHelp,
    Help(HelpKind),
    Open, // Implies REPL 
    Create, // Write file (no REPL)
    EmptyRepl, // Spawn a REPL not attached to any DB file
}
impl Default for Mode {
    fn default() -> Self {
        Mode::Help(HelpKind::default())
    }
}

impl From<Mode> for isize {
    fn from(mode: Mode) -> isize {
        match mode {
            Mode::HelpHelp => isize::MAX,
            Mode::Help(_) => 0_isize,
            Mode::Open => 1_isize,
            Mode::Create => 2_isize,
            Mode::EmptyRepl => 3_isize,
            _ => -1 as isize, 
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Copy)]
pub(crate) enum HelpKind {
    // Simple command usage
    #[default]
    SimpleHelp,
    
    // To prevent a circular dependency on Mode,
    // [`Mode::Help(HelpKind::ModeHelp(/* mode_num = */ isize))`]
    // will internally match the argument mode string to its corresponding
    // integer. If this match fails, a negative integer is supplied.
    // This means they probably misspelled the subcommand.
    // In the future, this allows a more refined client to assign a negative 
    // integer or span of negative integers to a set of most likely candidates
    // for a particular subcommand that was misspelled, leading to helpful
    // behavior like: "Did not recognize `relp`. Did you mean `repl`?"
    ModeHelp(isize),

    // How do I ask for help?
    // <cmd> help help # obviously
    HelpHelp,
}

impl<'a> From<&'a str> for HelpKind {
    fn from(it: &'a str) -> HelpKind {
        match it {
            "help" => HelpKind::ModeHelp(0_isize),
            "open" => HelpKind::ModeHelp(1_isize),
            "create" => HelpKind::ModeHelp(2_isize),
            "repl" => HelpKind::ModeHelp(3_isize),
            _ => HelpKind::ModeHelp(-1 as isize),
        }
    }
}

pub(crate) fn dispatch(op: Operation) -> anyhow::Result::<()> {
    match &op.mode {
        &Mode::Help(kind) => print_help(/* kind = */ HelpKind::from(kind)),
        &Mode::HelpHelp => print_helphelp(),
        &Mode::EmptyRepl => panic!("Not implemented: REPL"),
        &Mode::Create => try_create(op)?,
        &Mode::Open => panic!("Not implemented: REPL"),
    };
    
    Ok(())
}

fn print_help(kind: HelpKind) {
    println!("Hello");
}

fn print_helphelp() {
    println!("Manual");
}

fn try_create(op: Operation) -> anyhow::Result<()> {
    let path = &op.args.unwrap().0[0];
    let mut db = FunkDb::open(path)?;
    let _ = &mut db.save()?;
    Ok(())
}

#[cfg(test)]
mod clitest {
    use super::{dispatch, Operation, Mode, parse_cli, Args, Kwargs, HelpKind};
    use funk::FunkDb;
    
    #[test]
    fn cli_parses_new() {
        let expected_args: Args = Args(vec![String::from("test.funk")]);
        let expected_mode = Mode::Create;
        let expected_op = Operation::builder().mode(expected_mode).args(expected_args).build();

        use std::ffi::OsString;
        let given: Vec<OsString> = vec![OsString::from("create"), OsString::from("test.funk")];
        let actual: Operation = parse_cli(given);

        assert_eq!(expected_op, actual);        
    }

    #[test]
    fn prints_simple_help() {
        let expected_kwargs: Kwargs = Kwargs(vec![]);
        let expected_mode = Mode::Help(HelpKind::SimpleHelp);
        let expected_op = Operation::builder()
            .mode(expected_mode)
            .build();

        use std::ffi::OsString;
        let given: Vec<OsString> = vec![OsString::from("help")];
        let actual: Operation = parse_cli(given);

        assert_eq!(expected_op, actual);
    }
   
    #[test]
    fn parse_mode_help() {
        use std::ffi::OsString;
        fn works(given: &str) -> bool {
            let mut got: Vec<OsString> = vec![];
            for give in given.split(' ').into_iter() {
                &mut got.push(OsString::from(give));
            }
            parse_cli(got).mode == Mode::Help(HelpKind::SimpleHelp)
        }

        assert!(works("new database.txt"));
        assert!(works("helpzzzzzzzzzzz"));
        assert!(works("--"));
        assert!(works("new --help"));
    }

    #[ignore]
    #[test]
    fn spawn_repl() {
        todo!("Open a REPL programmatically, verifying text streamed along fake stdin");
    }

    #[ignore]
    #[test]
    fn open_existing_db() {
        todo!("Open canonicalized path as funkdb file");
    }

    #[test]
    fn new_database_file() {
        use std::ffi::OsString;
        let dbpath = "test.funk";
        let op = parse_cli(vec![OsString::from("create"), OsString::from("test.funk")]);
        let result = dispatch(op);
        assert!(result.is_ok());
        let found = std::path::Path::new(dbpath).try_exists();
        if found.is_err() { panic!("Failed to read file system"); }
        let found = found.unwrap();
        assert!(found);
        std::fs::remove_file(dbpath).unwrap();
    }    
}
