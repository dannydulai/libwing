mod utils; 
use utils::Args;

use std::result::Result;

use libwing::{WingConsole, WingResponse, WingNodeDef, NodeType};

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
    let proptype;
    let propparentid;

    fn parse_id(name: &str) -> (i32, i32, String, NodeType) {
        let propid;
        let propparentid;
        let propname;
        let proptype;

        if let Ok(id) = name.parse::<i32>() {
            propid = id;
            if let Some(defs) = WingConsole::id_to_defs(id) {
                if defs.len() == 1 {
                    proptype = defs[0].1.node_type;
                    propparentid = defs[0].1.parent_id;
                    propname = defs[0].0.clone();
                } else {
                    eprintln!("property id {} maps to multiple names, which may have different types. Use a full name please:", id);
                    eprintln!();
                    for (i, (name, _)) in defs.iter().enumerate() {
                        eprintln!("{}. {}", i+1, name);
                    }
                    eprintln!();
                    std::process::exit(1);
                }
            } else {
                eprintln!("invalid property id: {}", id);
                std::process::exit(1);
            }
        } else {
            propname = name.to_string();
            if let Some(def) = WingConsole::name_to_def(name) {
                propid = def.id;
                proptype = def.node_type;
                propparentid = def.parent_id;
            } else {
                eprintln!("invalid property name: {}", name);
                std::process::exit(1);
            }
        }
        (propid, propparentid, propname, proptype)
    }

    let action = 
        if arg.ends_with("?") {
            let name = arg.trim_end_matches("?");
            (propid, propparentid, propname, proptype) = parse_id(name);
            Action::Definition

        } else {
            let parts:Vec<&str> = arg.split("=").collect();
            if parts.len() == 2 {
                (propid, propparentid, propname, proptype) = parse_id(parts[0]);
                Action::Set(parts[1].to_string())
            } else if parts.len() == 1 {
                (propid, propparentid, propname, proptype) = parse_id(parts[0]);
                Action::Lookup
            } else {
                eprintln!("invalid argument. only 1 equals allowed.");
                std::process::exit(1);
            }
        };

    let mut wing = WingConsole::connect(host.as_deref())?;
    
    match action {
        Action::Lookup => {
            if proptype == NodeType::Node {
                wing.request_node_definition(propid)?;
            } else {
                wing.request_node_data(propid)?;
            }
        },
        Action::Set(val) => {
            match proptype {
                NodeType::Node => {
                    eprintln!("Can not set node {} because it's a node, and not a property.", propname);
                    std::process::exit(1);
                },
                NodeType::StringEnum |
                NodeType::String => {
                    wing.set_string(propid, &val)?;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                },
                NodeType::Integer => {
                    if let Ok(v) = val.parse::<i32>() {
                        wing.set_int(propid, v)?;
                    } else {
                        eprintln!("Property {} is an integer, but that was not passed: {}", propname, val);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    std::process::exit(0);
                },
                NodeType::FloatEnum |
                NodeType::FaderLevel |
                NodeType::LogarithmicFloat |
                NodeType::LinearFloat => {
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
            if proptype == NodeType::Node {
                wing.request_node_definition(propparentid)?;
            } else {
                wing.request_node_definition(propid)?;
            }
        }
    }

    let mut children = Vec::<WingNodeDef>::new();

    loop {
        match wing.read()? {
            WingResponse::RequestEnd => {
                if !children.is_empty() {
                    if jsonoutput {
                        let mut ret = jzon::array![ ];
                        for child in children {
                            ret.push(child.to_json()).unwrap();
                        }
                        println!("{}", ret);

                    } else {
                        for child in children {
                            println!("{}", child.to_description());
                            println!();
                        }
                    }
                }
                std::process::exit(0);
            },
            WingResponse::NodeData(_, id, data) => {
                if id == propid {
                    match proptype {
                        NodeType::Node => {
                            eprintln!("printing node for {}", propname);
                            std::process::exit(1);
                        },
                        NodeType::StringEnum |
                        NodeType::Integer |
                        NodeType::FloatEnum |
                        NodeType::LinearFloat |
                        NodeType::LogarithmicFloat |
                        NodeType::FaderLevel |
                        NodeType::String => {
                            if jsonoutput {
                                println!("{}", data.get_string());
                            } else {
                                println!("{} = {}", propname, data.get_string());
                            }
                        },
                    }
                }
            },
            WingResponse::NodeDef(d) => {
                if d.id == propid && matches!(action, Action::Definition) {
                    if jsonoutput {
                        let mut json = d.to_json();
                        json.insert("fullname", propname.clone()).unwrap();
                        println!("{}", json);
                    } else {
                        println!("Property:  {}", propname);
                        println!("{}", d.to_description());
                        println!();
                    }
                }
                if proptype == NodeType::Node && matches!(action, Action::Lookup) && d.parent_id == propid {
                    children.push(d);
                }
            },
        }
    }
}
