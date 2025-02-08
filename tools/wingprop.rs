use std::result::Result;
use libwing::{WingConsole, WingResponse, NodeType};

struct Args {
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
}

fn main() -> Result<(),libwing::Error> {
    let mut args = Args::new(r#"
Usage: wingprop [-h host] property[=value|?]

   -h host : IP address or hostname of Wing mixer. Default is to discover and connect to the first mixer found.
   -m      : Minimal output. Prints only the value or JSON of the definition.

   examples:
       wingprop /main/1/mute=1 # set a property
       wingprop /main/1/mute   # get a property's value
       wingprop /main/1/mute?  # get a property's definition

"#);
    let mut host = None;
    let mut minimal = false;

    let mut arg = args.next();
    if arg == "-h" { host = Some(args.next()); arg = args.next(); }
    if arg == "-m" { minimal = true; arg = args.next(); }

    #[derive(Debug)]
    enum Action {
        Lookup,
        Set(String),
        Definition,
    }

    let propname;
    let propid;
  
    let action = 
    if arg.ends_with("?") {
        propname = arg.trim_end_matches("?").to_string();
        propid = WingConsole::name_to_id(&propname); 

        if propid == 0 {
            eprintln!("invalid property name: {}", propname);
            std::process::exit(1);
        }
        Action::Definition

    } else {
        let parts:Vec<&str> = arg.split("=").collect();
        if parts.len() == 2 {
            propname = parts[0].to_string();
            propid = WingConsole::name_to_id(&propname);
            if propid == 0 {
                eprintln!("invalid property name: {}", &propname);
                std::process::exit(1);
            }
            Action::Set(parts[1].to_string())
        } else if parts.len() == 1 {
            propname = parts[0].to_string();
            propid = WingConsole::name_to_id(&propname);
            if propid == 0 {
                eprintln!("invalid property name: {}", &propname);
                std::process::exit(1);
            }
            Action::Lookup
        } else { eprintln!("invalid argument. only 1 equals allowed.");
            std::process::exit(1);
        }
    };


    if host.is_none() {
        // Discover Wing devices
        let devices = WingConsole::scan(true)?;
        if !devices.is_empty() { host = Some(devices[0].ip.clone()) }
    }

    if host.is_none() {
        eprintln!("No Wing devices found!");
        std::process::exit(1);
    }

    let host = host.unwrap();

    let mut console = WingConsole::connect(&host)?;
    
    match action {
        Action::Lookup => {
            console.request_node_data(propid)?;
        },
        Action::Set(val) => {
            match WingConsole::id_to_type(propid) {
                None => {
                    eprintln!("Unknown property type for {}", propname);
                    std::process::exit(1);
                },
                Some(NodeType::Node) => {
                    eprintln!("trying to set {}, but it's a node", propname);
                    std::process::exit(1);
                },
                Some(NodeType::StringEnum) |
                Some(NodeType::String) => {
                    console.set_string(propid, &val)?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                },
                Some(NodeType::Integer) => {
                    if let Ok(v) = val.parse::<i32>() {
                        console.set_int(propid, v)?;
                    } else {
                        eprintln!("Property {} is an integer, but that was not passed: {}", propname, val);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                },
                Some(NodeType::FloatEnum) |
                Some(NodeType::FaderLevel) |
                Some(NodeType::LogarithmicFloat) |
                Some(NodeType::LinearFloat) => {
                    if let Ok(v) = val.parse::<f32>() {
                        console.set_float(propid, v)?;
                    } else {
                        eprintln!("Property {} is a floating point number, but that was not passed: {}", propname, val);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                }
            }
        },
        Action::Definition => {
            console.request_node_definition(propid)?;
        }
    }

    // Main event loop
    loop {
        match console.read()? {
            WingResponse::RequestEnd => {
            },
            WingResponse::NodeData(id, data) => {
                println!("{:08X} = {}", id, data.get_string());
                if id == propid {
                    match WingConsole::id_to_type(id) {
                        None => {
                            eprintln!("Unknown property type for {}", propname);
                            std::process::exit(1);
                        },
                        Some(NodeType::Node) => {
                            // println!("{} = {}", propname, data.get_string());
                            eprintln!("printing node for {}", propname);
                            std::process::exit(0);
                        },
                        Some(NodeType::StringEnum) |
                        Some(NodeType::String) => {
                            if minimal {
                                println!("{}", data.get_string());
                            } else {
                                println!("{} = {}", propname, data.get_string());
                            }
                            std::process::exit(0);
                        },
                        Some(NodeType::Integer) => {
                            if minimal {
                                println!("{}", data.get_int());
                            } else {
                                println!("{} = {}", propname, data.get_int());
                            }
                            std::process::exit(0);
                        },
                        Some(NodeType::FloatEnum) |
                        Some(NodeType::LinearFloat) |
                        Some(NodeType::LogarithmicFloat) |
                        Some(NodeType::FaderLevel) => {
                            if minimal {
                                println!("{}", data.get_float());
                            } else {
                                println!("{} = {}", propname, data.get_float());
                            }
                            std::process::exit(0);
                        }
                    }
                }
            },
            WingResponse::NodeDef(d) => {
                println!("<{:08X}> => {}", d.id, d.to_json());
                if d.id == propid {
                    if minimal {
                        println!("{}", d.to_json());
                    } else {
                        println!("{}", d.to_description());
                    }
                    std::process::exit(0);
                }
            },
        }
    }
}
