use std::io::{self, Write};
use wing::{WingConsole, NodeDefinition, NodeData};

fn main() -> wing::Result<()> {
    // Discover Wing devices
    let devices = WingConsole::scan(true);
    if devices.is_empty() {
        eprintln!("No Wing devices found!");
        return Ok(());
    }

    println!("Found Wing at {}", devices[0].ip);
    println!("Connecting...");
    
    let mut console = WingConsole::connect(&devices[0].ip)?;
    println!("Connected!");
    
    let mut stdout = io::stdout();
    
    console.on_node_definition = Some(Box::new(move |def: NodeDefinition| {
        writeln!(stdout, "Node Definition:").unwrap();
        writeln!(stdout, "  ID: {:06X}", def.id).unwrap();
        writeln!(stdout, "  Name: {}", def.name).unwrap();
        writeln!(stdout, "  Type: {:?}", def.node_type).unwrap();
        writeln!(stdout, "  Unit: {:?}", def.unit).unwrap();
        stdout.flush().unwrap();
    }));
    
    console.on_node_data = Some(Box::new(move |id: u32, data: NodeData| {
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
