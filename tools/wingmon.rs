mod utils; 
use utils::Args;

use std::result::Result;

use libwing::{WingConsole, WingResponse};

fn main() -> Result<(),libwing::Error> {
    let mut args = Args::new(r#"
Usage: wingmon [-h host]

   -h host : IP address or hostname of Wing mixer. Default is to discover and connect to the first mixer found.
"#);
    let mut host = None;
    if args.has_next() && args.next() == "-h" { host = Some(args.next()); }

    let mut wing = WingConsole::connect(host.as_deref())?;
    println!("Connected!");

    loop {
        if let WingResponse::NodeData(_, id, data) =  wing.read()? {
            match WingConsole::id_to_defs(id) {
                None => println!("<Unknown:{}> = {}", id, data.get_string()),
                Some(defs) if defs.is_empty() => println!("<Unknown:{}> = {}", id, data.get_string()),
                Some(defs) if defs.len() == 1 => {
                    println!("{} = {}", defs[0].0, data.get_string());
                }
                Some(defs) if (defs.len() > 1) => {
                    let u = std::collections::HashSet::<u16>::from_iter(defs.iter().map(|(_, def)| def.index));
                    if u.len() == 1 {
                        // let propname = String::from("/") + &defs[0].0.split("/").enumerate().filter(|(i, _)| *i < defs.len()-1).map(|(_, n)| n).collect::<Vec<_>>().join("/") +
                        let propname = String::from("prop") + defs[0].1.index.to_string().as_str();
                        println!("{} = {} (check out propmap.jsonl for more info on property with id {})", propname, data.get_string(), id);
                    } else {
                        println!("<MultiProp:{}> = {} (check out propmap.jsonl for more info on property id {})", id, data.get_string(), id);
                    }
                }
                Some(_) => {}

            }
        }
    }
}
