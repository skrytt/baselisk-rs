extern crate rustyline;

use defs;

use rustyline::completion::{Completer, extract_word};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, Helper, KeyPress};

use cli::tree::Tree;
use comms::MainThreadComms;

static BREAK_CHARS: [u8; 1] = [b' '];

#[cfg(unix)]
static ESCAPE_CHAR: Option<char> = Some('\\');

#[cfg(windows)]
static ESCAPE_CHAR: Option<char> = None;

struct CliHelper {
    tree: Tree,
}

impl CliHelper {
    fn new(tree: Tree) -> CliHelper {
        CliHelper {
            tree
        }
    }
}

impl Completer for CliHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
    ) -> Result<(usize, Vec<String>), ReadlineError> {
        let (start, partial) = extract_word(line, pos, ESCAPE_CHAR, &BREAK_CHARS);

        let mut result_vec = Vec::new();

        if let Ok(options) = self.tree.get_completion_options(line) {
            let matches = options
                .into_iter()
                .filter(|s| { s.starts_with(partial) });
            for item in matches {
                result_vec.push(item);
            }
        }

        Ok((start, result_vec))
    }
}

impl Hinter for CliHelper {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }
}

impl Highlighter for CliHelper {}

impl Helper for CliHelper {}

pub struct Cli {
    rl: Editor<CliHelper>,
    comms: MainThreadComms,
}

impl Cli {
    pub fn new(tree: Tree, comms: MainThreadComms) -> Cli {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .output_stream(OutputStreamType::Stdout)
            .build();

        let h = CliHelper::new(tree);

        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(h));
        rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
        rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
        Cli {
            rl,
            comms
        }
    }

    pub fn read_until_interrupted(&mut self) {
        loop {
            let readline = self.rl.readline(defs::PROMPT);
            match readline {
                Ok(line) => {
                    self.rl.add_history_entry(line.as_ref());
                    println!("Line: {}", line);
                    if let Some(helper) = self.rl.helper() {
                        helper.tree.execute_command(line, &mut self.comms);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}

