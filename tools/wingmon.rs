use std::io::{self, Write};
use libwing::{WingConsole, WingResponse};
use std::result::Result;

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

    println!("Connecting to {} at {}...", device.name, device.ip);
    let mut console = WingConsole::connect(&device.ip)?;
    println!("Connected!");

    loop {
        if let WingResponse::NodeData(id, data) =  console.read()? {
            println!("{} {} = {}",
                id,
                WingConsole::id_to_name(id).unwrap_or(&format!("<Unknown:{}>", id)),
                data.get_string());
        }
    }
}
