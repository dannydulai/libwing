use std::io::{self, Write};
use wing::{WingConsole, NodeDefinition, NodeData};

fn main() -> wing::Result<()> {
    // Discover Wing devices
    let devices = WingConsole::scan(false);
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
    
    let stdout = io::stdout();
    let stdout = std::sync::Mutex::new(stdout);
    let stdout1 = std::sync::Arc::new(stdout);
    let stdout2 = stdout1.clone();
    
    console.on_node_definition = Some(Box::new(move |def: NodeDefinition| {
        let mut stdout = stdout1.lock().unwrap();
        writeln!(stdout, "Node Definition:").unwrap();
        writeln!(stdout, "  ID: {:06X}", def.id).unwrap();
        writeln!(stdout, "  Name: {}", def.name).unwrap();
        writeln!(stdout, "  Type: {:?}", def.node_type).unwrap();
        writeln!(stdout, "  Unit: {:?}", def.unit).unwrap();
        stdout.flush().unwrap();
    }));
    
    console.on_node_data = Some(Box::new(move |id: u32, data: NodeData| {
        let mut stdout = stdout2.lock().unwrap();
        write!(stdout, "Node {:06X} = ", id).unwrap();
        if data.has_string() {
            writeln!(stdout, "{}", data.get_string()).unwrap();
        } else if data.has_float() {
            writeln!(stdout, "{}", data.get_float()).unwrap();
        } else if data.has_int() {
            writeln!(stdout, "{}", data.get_int()).unwrap();
        }
        stdout.flush().unwrap();
    }));

    // Main event loop
    loop {
        console.read()?;
    }
}
