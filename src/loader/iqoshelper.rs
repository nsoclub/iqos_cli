use rustyline::completion::{Completer, Pair};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::Context;
use rustyline::Helper;
use rustyline::error::ReadlineError;

pub struct IqosHelper {
    commands: Vec<String>,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
}

impl IqosHelper {
    pub fn new() -> Self {
        let commands = vec![
            "brightness".to_string(),
            "findmyiqos".to_string(),
            "smartgesture".to_string(),
            "vibration".to_string(),
            "help".to_string(),
            "exit".to_string(),
            "info".to_string(),
        ];
        
        IqosHelper {
            commands,
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
        }
    }
}

impl Completer for IqosHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let args: Vec<&str> = line[..pos].split_whitespace().collect();
        
        if args.is_empty() {
            // 入力がない場合は全コマンドを提案
            let candidates: Vec<Pair> = self.commands
                .iter()
                .map(|cmd| Pair { display: cmd.clone(), replacement: cmd.clone() })
                .collect();
            
            return Ok((0, candidates));
        }
        
        if args.len() == 1 {
            // 最初の引数の補完
            let current = args[0];
            let start = line.len() - current.len();
            
            let candidates: Vec<Pair> = self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(current))
                .map(|cmd| Pair { display: cmd.clone(), replacement: cmd.clone() })
                .collect();
            
            return Ok((start, candidates));
        }
        
        if args.len() == 2 {
            // サブコマンドの補完
            let cmd = args[0];
            let subcmd = args[1];
            let start = line.len() - subcmd.len();
            
            let candidates = match cmd {
                "brightness" => vec!["high", "medium", "low"]
                    .iter()
                    .filter(|sc| sc.starts_with(subcmd))
                    .map(|sc| Pair { display: sc.to_string(), replacement: sc.to_string() })
                    .collect(),
                    
                "smartgesture" => vec!["enable", "disable"]
                    .iter()
                    .filter(|sc| sc.starts_with(subcmd))
                    .map(|sc| Pair { display: sc.to_string(), replacement: sc.to_string() })
                    .collect(),
                
                "vibration" => vec!["charge", "heating", "starting", "terminated", "puffend"]
                    .iter()
                    .filter(|sc| sc.starts_with(subcmd))
                    .map(|sc| Pair { display: sc.to_string(), replacement: sc.to_string() })
                    .collect(),
                    
                _ => vec![],
            };
            
            return Ok((start, candidates));
        }
        
        if args.len() == 3 && args[0] == "vibration" {
            let option_value = args[2];
            let start = line.len() - option_value.len();
            
            let candidates: Vec<Pair> = vec!["on", "off"]
                .iter()
                .filter(|val| val.starts_with(option_value))
                .map(|val| Pair { display: val.to_string(), replacement: val.to_string() })
                .collect();
                
            return Ok((start, candidates));
        }
        
        Ok((pos, vec![]))
    }
}

// HelperトレイトおよびHelper関連トレイトの実装
impl Helper for IqosHelper {}

impl Hinter for IqosHelper {
    type Hint = String;
    
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for IqosHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        self.highlighter.highlight_prompt(prompt, default)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        self.highlighter.highlight_hint(hint)
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for IqosHelper {}
