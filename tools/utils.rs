pub struct Args {
    help: String,
    args: Vec<String>,
    it:   u8,
}

impl Args {
    pub fn new(help: &str) -> Self {
        Self {
            help: help.to_string(),
            args: std::env::args().skip(1).collect(),
            it: 0,
        }
    }

    pub fn print_help(&mut self, msg: Option<&str>) {
        if let Some(msg) = msg {
            eprintln!("{}", msg);
            eprintln!();
        }
        eprintln!("{}", self.help);
    }

    pub fn next(&mut self) -> String {
        if self.it >= self.args.len() as u8 {
            self.print_help(None);
            std::process::exit(1);
        }
        self.it += 1;
        self.args[(self.it-1) as usize].clone()
    }

    #[allow(dead_code)]
    pub fn has_next(&self) -> bool {
        self.it < self.args.len() as u8
    }
}

