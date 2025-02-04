use std::io::{self, Write};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use wing::{WingConsole, NodeDefinition, NodeType};

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
    let console = Arc::new(Mutex::new(WingConsole::connect(&device.ip)?));
    println!("Connected!");

    // Track parent-child relationships and node definitions
    let node_parent_to_children = Arc::new(Mutex::new(HashMap::<i32, Vec<i32>>::new()));
    let node_id_to_def = Arc::new(Mutex::new(HashMap::<i32, NodeDefinition>::new()));
    let pending_requests = Arc::new(AtomicI32::new(0));

    let node_parent_to_children2 = node_parent_to_children.clone();
    let node_id_to_def2 = node_id_to_def.clone();
    let pending_requests2 = pending_requests.clone();
    let console2 = console.clone();

    {
        let mut console = console.lock().unwrap();
        console.on_node_definition = Some(Box::new(move |def: NodeDefinition| {
            // Store the node definition
            let node_id = def.id;
            let parent_id = def.parent_id;
            
            // Update parent-child relationship
            node_parent_to_children2.lock().unwrap()
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(node_id);
                
            // Store node definition
            node_id_to_def2.lock().unwrap()
                .insert(node_id, def.clone());
        
        // Print progress
        print!("\rReceived {} properties", node_id_to_def2.lock().unwrap().len());
        io::stdout().flush().unwrap();

        // For node types that can have children, request their definitions
        if def.node_type == NodeType::Node {
            // Request definitions for potential child nodes
            for i in 1..=64 {
                let child_id = (node_id << 8) | i;
                console2.lock().unwrap().request_node_definition(child_id).unwrap();
                pending_requests2.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Format and output JSON
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
            NodeType::LinearFloat | 
            NodeType::LogarithmicFloat |
            NodeType::FaderLevel => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "min_float": def.min_float,
                    "max_float": def.max_float,
                    "steps": def.steps,
                }).as_object().unwrap().clone());
            }
            NodeType::Integer => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "min_int": def.min_int,
                    "max_int": def.max_int,
                }).as_object().unwrap().clone());
            }
            NodeType::String => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "max_string_len": def.max_string_len,
                }).as_object().unwrap().clone());
            }
            NodeType::StringEnum => {
                json.as_object_mut().unwrap().extend(serde_json::json!({
                    "items": def.string_enum.iter().map(|item| {
                        serde_json::json!({
                            "item": item.item,
                            "longitem": item.longitem,
                        })
                    }).collect::<Vec<_>>(),
                }).as_object().unwrap().clone());
            }
            NodeType::FloatEnum => {
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

        pending_requests2.fetch_sub(1, Ordering::SeqCst);
    }));

    // Start with root node
    console.lock().unwrap().request_node_definition(0)?;
    pending_requests.fetch_add(1, Ordering::SeqCst);
    
    // Process responses until we've handled all requests
    while pending_requests.load(Ordering::SeqCst) > 0 {
        console.lock().unwrap().read()?;
    }

    println!("\nSchema retrieval complete!");

    Ok(())
}
