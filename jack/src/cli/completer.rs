extern crate rustyline;

use defs;

use rustyline::completion::{Completer, extract_word, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, Helper, KeyPress};

use baselisk_core::shared::SharedState as SharedState;
use cli::tree::Tree;
use std::sync::Arc;
use std::fs::File;
use std::io::{BufRead, BufReader};

static BREAK_CHARS: [u8; 1] = [b' '];

#[cfg(unix)]
static ESCAPE_CHAR: Option<char> = Some('\\');

#[cfg(windows)]
static ESCAPE_CHAR: Option<char> = None;

struct CliHelper {
    tree: Tree,
}

impl CliHelper {
    fn new(tree: Tree) -> Self {
        Self {
            tree
        }
    }
}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let (start, partial) = extract_word(line, pos, ESCAPE_CHAR, &BREAK_CHARS);

        let mut result_vec: Vec<Pair> = Vec::new();

        if let Ok(options) = self.tree.get_completion_options(line) {
            let matches = options
                .into_iter()
                .filter(|s| { s.replacement.starts_with(partial) });
            for item in matches {
                result_vec.push(item);
            }
        }

        Ok((start, result_vec))
    }
}

impl Hinter for CliHelper {
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CliHelper {}

impl Helper for CliHelper {}

pub struct Cli {
    rl: Editor<CliHelper>,
    shared_state: Arc<SharedState>,
}

impl Cli {
    pub fn new(tree: Tree,
               shared_state: Arc<SharedState>,
    ) -> Self {
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
        Self {
            rl,
            shared_state
        }
    }

    pub fn read_from_file(&mut self, file_path: &str) -> std::io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines = reader.lines();
        for line in lines {
            match line {
                Err(_) => {
                    println!("Error parsing input file");
                    break
                },
                Ok(line) => {
                    if let Some(helper) = self.rl.helper() {
                        helper.tree.execute_command(line.as_str(), &self.shared_state);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn read_until_interrupted(&mut self) {
        loop {
            let readline = self.rl.readline(defs::PROMPT);
            match readline {
                Ok(line) => {
                    self.rl.add_history_entry(line.as_str());
                    if let Some(helper) = self.rl.helper() {
                        helper.tree.execute_command(line.as_str(), &self.shared_state);
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

