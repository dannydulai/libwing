use std::io::{self, Write};
use wing::{WingConsole, NodeDefinition};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <map_file> [node_id]", args[0]);
        return;
    }

    if let Err(e) = NodeDefinition::init_map(&args[1]) {
        eprintln!("Failed to load map file: {}", e);
        return;
    }

    if args.len() > 2 {
        if let Ok(id) = u32::from_str_radix(&args[2], 16) {
            print_node(id);
        }
    } else {
        // Print root node
        print_node(0);
    }
}

fn print_node(id: u32) {
    let ip = "192.168.1.1"; // Default IP
    let mut console = match WingConsole::connect(ip) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    let mut stdout = io::stdout();
    
    console.on_node_definition = Some(Box::new(move |def: NodeDefinition| {
        let mut json = serde_json::json!({
            "id": format!("{:06X}", def.id),
            "parent_id": format!("{:06X}", def.parent_id),
            "name": def.name,
            "long_name": def.long_name,
            "type": format!("{:?}", def.node_type),
            "unit": format!("{:?}", def.unit),
            "read_only": def.read_only,
        });

        match def.node_type {
            wing::NodeType::LinearFloat | 
            wing::NodeType::LogarithmicFloat |
            wing::NodeType::FaderLevel => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "min_float": def.min_float,
                    "max_float": def.max_float,
                    "steps": def.steps,
                }).as_object().unwrap().clone());
            }
            wing::NodeType::Integer => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "min_int": def.min_int,
                    "max_int": def.max_int,
                }).as_object().unwrap().clone());
            }
            wing::NodeType::String => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "max_string_len": def.max_string_len,
                }).as_object().unwrap().clone());
            }
            wing::NodeType::StringEnum => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "items": def.string_enum.iter().map(|item| {
                        serde_json::json!({
                            "item": item.item,
                            "longitem": item.longitem,
                        })
                    }).collect::<Vec<_>>(),
                }).as_object().unwrap().clone());
            }
            wing::NodeType::FloatEnum => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "items": def.float_enum.iter().map(|item| {
                        serde_json::json!({
                            "item": item.item,
                            "longitem": item.longitem,
                        })
                    }).collect::<Vec<_>>(),
                }).as_object().unwrap().clone());
            }
            _ => {}
        }
        writeln!(stdout, "{}", serde_json::to_string(&json).unwrap()).unwrap();
        stdout.flush().unwrap();
    }));

    console.request_node_definition(id);
    
    // Read responses for a short time
    for _ in 0..10 {
        if let Err(e) = console.read() {
            eprintln!("Error reading: {}", e);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
