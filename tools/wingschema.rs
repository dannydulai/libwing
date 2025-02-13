use std::fs::File;
use std::io::{self, Write};
use std::sync::RwLock;
use std::result::Result;
use std::collections::HashMap;
use libwing::{WingConsole, WingResponse, WingNodeDef, NodeType};

lazy_static::lazy_static! {
    static ref node_parent_to_children: RwLock<HashMap::<i32, Vec<i32>>> = RwLock::new(HashMap::<i32, Vec<i32>>::new());
    static ref node_id_to_def: RwLock<HashMap::<i32, WingNodeDef>> = RwLock::new(HashMap::<i32, WingNodeDef>::new());
}

fn req(node_id: i32, wing: &mut WingConsole) -> i32 {
    let mut done = 0;

    if ! node_parent_to_children.read().unwrap().contains_key(&node_id) {
        node_parent_to_children.write().unwrap().insert(node_id, Vec::<i32>::new());
        done += 1;
        wing.request_node_definition(node_id).unwrap();
        if done >= 100 { return done; }
    }

    let h = node_id_to_def.read().unwrap();
    if done == 0 {
        let children = node_parent_to_children.read().unwrap().get(&node_id).unwrap().clone();
        for child in children {
            let def = h.get(&child).unwrap();
            let has_child = node_parent_to_children.read().unwrap().contains_key(&child);
            if def.node_type == NodeType::Node && ! has_child {
                node_parent_to_children.write().unwrap().insert(child, Vec::<i32>::new());
                done += 1;
                wing.request_node_definition(child).unwrap();
                if done >= 100 { return done; }
            }
        }
    }

    if done == 0 {
        let children = node_parent_to_children.read().unwrap().get(&node_id).unwrap().clone();
        for child in children {
            let def = h.get(&child).unwrap();
            if def.node_type == NodeType::Node {
                done += req(child, wing);
                if done >= 100 { return done; }
            }
        }
    }

    done
}

fn print_node(json_file: &mut File, rust_file: &mut Vec<u8>, node_id: i32, recurs: bool) {
    if node_id != 0 {
        let h = node_id_to_def.read().unwrap();
        let def = h.get(&node_id).unwrap();

        let mut json = def.to_json();

        // the json is good here, but the fullname is based on the propmap.rs file, which this is
        // creating... so let's just build it up from scratch ourselves and replace the given
        // fullname

        let mut n = def;
        let mut fullname : String;
        if n.name.is_empty() {
            fullname = n.index.to_string();
        } else {
            fullname = n.name.clone();
        }
        while n.parent_id != 0 {
            if !h.contains_key(&n.parent_id) {
                fullname = "???/".to_owned() + &fullname[..];
                break;
            } else {
                n = h.get(&n.parent_id).unwrap();
                if n.name.is_empty() {
                    fullname = n.index.to_string() + "/" + &fullname[..];
                } else {
                    fullname = n.name.clone() + "/" + &fullname[..];
                }
            }
        }
        if n.parent_id == 0 { fullname = "/".to_owned() + &fullname[..]; }
        json.insert("fullname", fullname.clone()).unwrap();

        writeln!(json_file, "{}", jzon::stringify(json)).unwrap();

        for b in def.id.to_be_bytes() { rust_file.push(b); }
        for b in def.parent_id.to_be_bytes() { rust_file.push(b); }
        rust_file.push(def.node_type as u8);
        for b in (fullname.len() as u16).to_be_bytes() { rust_file.push(b); }
        for b in fullname.into_bytes() { rust_file.push(b); }
    }

    if recurs {
        if let Some(children) = node_parent_to_children.read().unwrap().get(&node_id) {
            for child in children {
                print_node(json_file, rust_file, *child, true);
            }
        }
    }
}

fn main() -> Result<(),libwing::Error> {
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

    print!("Connecting to {} at {}...", device.name, device.ip);
    io::stdout().flush()?;
    let mut wing = WingConsole::connect(&device.ip)?;
    println!("connected!");

    // Track parent-child relationships and node definitions
    let mut pending_requests = 0;
    let mut end_requests = 0;

    // Start with root node
    wing.request_node_definition(0)?;
    pending_requests += 1;

    // Process responses until we've handled all requests
    loop { 
        match wing.read()? {
            WingResponse::NodeData(_,_) => { }
            WingResponse::NodeDef(def) => {
                // Store the node definition
                let node_id = def.id;
                let parent_id = def.parent_id;

                // Update parent-child relationship
                node_parent_to_children.write().unwrap()
                    .entry(parent_id)
                    .or_default()
                    .push(node_id);

                // Store node definition
                node_id_to_def.write().unwrap().insert(node_id, def);

                // Print progress
                print!("\rReceived {} nodes", node_id_to_def.read().unwrap().len());
                io::stdout().flush().unwrap();
            }
            WingResponse::RequestEnd => {
                end_requests += 1;
//                println!("\nReceived request end, pending: {}, end: {}", pending_requests, end_requests);
                if end_requests == pending_requests {
                    let v = req(0, &mut wing);
                    pending_requests += v;
                    if v == 0 {
                        println!("Schema retrieval complete. {} records received.", node_id_to_def.read().unwrap().len());
                        println!("Writing schema files (propmap.jsonl and propmap.rs)...");
                        let mut json_file = std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open("propmap.jsonl")
                            .unwrap();

                        let mut rust_file = std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open("propmap.rs")
                            .unwrap();

                        writeln!(rust_file, "use std::collections::HashMap;").unwrap();
                        writeln!(rust_file, "lazy_static::lazy_static! {{").unwrap();
                        let mut vec = Vec::<u8>::new();
                        print_node(&mut json_file, &mut vec, 0, true);
                        writeln!(rust_file, "    pub static ref ID_TO_DATA: HashMap<i32, (String, i32, u8)> = {{").unwrap();
                        writeln!(rust_file, "        let mut m = HashMap::new();").unwrap();
                        write!(  rust_file, "        let d = b\"").unwrap();
                        for b in vec { write!(rust_file, "\\x{:02X}", b).unwrap(); }
                        writeln!(rust_file, "\";").unwrap();
                        writeln!(rust_file, "        let mut i = 0;").unwrap();
                        writeln!(rust_file, "        while i < d.len() {{").unwrap();
                        writeln!(rust_file, "            let id = i32::from_be_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);").unwrap();
                        writeln!(rust_file, "            i += 4;").unwrap();
                        writeln!(rust_file, "            let parent = i32::from_be_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);").unwrap();
                        writeln!(rust_file, "            i += 4;").unwrap();
                        writeln!(rust_file, "            let ty = d[i];").unwrap();
                        writeln!(rust_file, "            i += 1;").unwrap();
                        writeln!(rust_file, "            let len = i16::from_be_bytes([d[i], d[i + 1]]) as usize;").unwrap();
                        writeln!(rust_file, "            i += 2;").unwrap();
                        writeln!(rust_file, "            let name = String::from_utf8(d[i..i + len].to_vec()).unwrap();").unwrap();
                        writeln!(rust_file, "            i += len;").unwrap();
                        writeln!(rust_file, "            m.insert(id, (name, parent, ty));").unwrap();
                        writeln!(rust_file, "        }}").unwrap();
                        writeln!(rust_file, "        m").unwrap();
                        writeln!(rust_file, "    }};").unwrap();
                        writeln!(rust_file, "}}").unwrap();
                        break;
                    }
                }
            }
        }
    }

    println!("Done.");
    Ok(())
}
