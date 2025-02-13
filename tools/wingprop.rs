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
Usage: wingprop [-h host] [-j] property[=value|?]

   -h host : IP address or hostname of Wing mixer. Default is to discover and connect to the first mixer found.
   -j      : Prints JSON of the value or definition.

   examples:
       wingprop /main/1/mute=1 # set a property
       wingprop /main/1/mute   # get a property's value
       wingprop /main/1/mute?  # get a property's definition

"#);
    let mut host = None;
    let mut jsonoutput = false;

    let mut arg = args.next();
    if arg == "-h" { host = Some(args.next()); arg = args.next(); }
    if arg == "-j" { jsonoutput = true; arg = args.next(); }

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
            let name = arg.trim_end_matches("?");
            if let Some(id) = WingConsole::parse_id(name, false) {
                propname = id.0;
                propid = id.1;
            } else {
                eprintln!("invalid property name: {}", name);
                std::process::exit(1);
            }
            Action::Definition

        } else {
            let parts:Vec<&str> = arg.split("=").collect();
            if parts.len() == 2 {
                if let Some(id) = WingConsole::parse_id(parts[0], false) {
                    propname = id.0;
                    propid = id.1;
                } else {
                    eprintln!("invalid property name: {}", parts[0]);
                    std::process::exit(1);
                }
                Action::Set(parts[1].to_string())
            } else if parts.len() == 1 {
                if let Some(id) = WingConsole::parse_id(parts[0], false) {
                    propname = id.0;
                    propid = id.1;
                } else {
                    eprintln!("invalid property name: {}", parts[0]);
                    std::process::exit(1);
                }
                Action::Lookup
            } else {
                eprintln!("invalid argument. only 1 equals allowed.");
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

    let mut wing = WingConsole::connect(&host.unwrap())?;
    
    let proptype = WingConsole::id_to_type(propid);
    match action {
        Action::Lookup => {
            if proptype == Some(NodeType::Node) {
                wing.request_node_definition(propid)?;
            } else {
                wing.request_node_data(propid)?;
            }
        },
        Action::Set(val) => {
            match proptype {
                None => {
                    eprintln!("Property {} type could not be determined.", propname);
                    std::process::exit(1);
                },
                Some(NodeType::Node) => {
                    eprintln!("Can not set node {} because it's a node, and not a property.", propname);
                    std::process::exit(1);
                },
                Some(NodeType::StringEnum) |
                Some(NodeType::String) => {
                    wing.set_string(propid, &val)?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                },
                Some(NodeType::Integer) => {
                    if let Ok(v) = val.parse::<i32>() {
                        wing.set_int(propid, v)?;
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
                        wing.set_float(propid, v)?;
                    } else {
                        eprintln!("Property {} is a floating point number, but that was not passed: {}", propname, val);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                }
            }
        },
        Action::Definition => {
            if proptype == Some(NodeType::Node) {
                let parent = WingConsole::id_to_parent(propid).unwrap();
                wing.request_node_definition(parent)?;
            } else {
                wing.request_node_definition(propid)?;
            }
        }
    }

    let mut children = Vec::<String>::new();

    // Main event loop
    loop {
        match wing.read()? {
            WingResponse::RequestEnd => {
                if !children.is_empty() {
                    if jsonoutput {
                        let mut ret = jzon::array![ ];
                        for child in children {
                            ret.push(child).unwrap();
                        }
                        println!("{}", ret);

                    } else {
                        for child in children {
                            println!("{}", child);
                        }
                    }
                }
                std::process::exit(0);
            },
            WingResponse::NodeData(id, data) => {
//                println!("{} = {}", id, data.get_string());
                if id == propid {
                    match proptype {
                        None => {
                            eprintln!("Property {} type could not be determined.", propname);
                            std::process::exit(1);
                        },
                        Some(NodeType::Node) => {
                            // println!("{} = {}", propname, data.get_string());
                            eprintln!("printing node for {}", propname);
//                            std::process::exit(0);
                        },
                        Some(NodeType::StringEnum) |
                        Some(NodeType::String) => {
                            if jsonoutput {
                                println!("{}", data.get_string());
                            } else {
                                println!("{} = {}", propname, data.get_string());
                            }
//                            std::process::exit(0);
                        },
                        Some(NodeType::Integer) => {
                            if jsonoutput {
                                println!("{}", data.get_int());
                            } else {
                                println!("{} = {}", propname, data.get_int());
                            }
//                            std::process::exit(0);
                        },
                        Some(NodeType::FloatEnum) |
                        Some(NodeType::LinearFloat) |
                        Some(NodeType::LogarithmicFloat) |
                        Some(NodeType::FaderLevel) => {
                            if jsonoutput {
                                println!("{}", data.get_float());
                            } else {
                                println!("{} = {}", propname, data.get_float());
                            }
//                            std::process::exit(0);
                        }
                    }
                }
            },
            WingResponse::NodeDef(d) => {
 //               println!("<{}> => {}", d.id, d.to_json());
                if d.id == propid && matches!(action, Action::Definition) {
                    if jsonoutput {
                        println!("{}", d.to_json());
                    } else {
                        println!("{}", d.to_description());
                    }
//                    std::process::exit(0);
                }
                if proptype == Some(NodeType::Node) && matches!(action, Action::Lookup) && d.parent_id == propid {
                    children.push(d.name.clone());
                }
            },
        }
    }
}
