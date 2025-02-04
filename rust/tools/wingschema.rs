use std::io::{self, Write};
use wing::{WingConsole, NodeDefinition};

fn main() -> wing::Result<()> {
    // Discover Wing devices
    let devices = WingConsole::scan(true)?;
    if devices.is_empty() {
        eprintln!("No Wing devices found!");
        return Ok(());
    }

    // Print discovered devices
    println!("Found {} Wing device(s):", devices.len());
    for (i, dev) in devices.iter().enumerate() {
        println!("{}. {} at {} (Model: {}, Firmware: {})", 
            i + 1, dev.name, dev.ip, dev.model, dev.firmware);
    }

    // Let user choose if multiple devices found
    let device = if devices.len() > 1 {
        print!("Select device (1-{}): ", devices.len());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input.trim().parse::<usize>().unwrap_or(0);
        if selection < 1 || selection > devices.len() {
            eprintln!("Invalid selection!");
            return Ok(());
        }
        &devices[selection - 1]
    } else {
        &devices[0]
    };

    println!("Connecting to {} at {}...", device.name, device.ip);
    let mut console = WingConsole::connect(&device.ip)?;
    println!("Connected!");

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

    console.request_node_definition(0)?;
    
    loop {
        console.read()?;
    }
}
