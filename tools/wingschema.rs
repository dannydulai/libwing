mod utils; 
use utils::Args;

use std::fs::File;
use std::io::Write;
use std::result::Result;

use libwing::{WingConsole, WingResponse, WingNodeDef};

fn get_node_def(wing: &mut WingConsole, parents: Vec<i32>) -> Vec<Vec<WingNodeDef>> {
    for parent in &parents {
        wing.request_node_definition(*parent).unwrap();
    }

    let mut ret = Vec::new();
    for parent in &parents {
        let mut ret2 = Vec::new();
        loop {
            match wing.read().unwrap() {
                WingResponse::NodeData(_,_,_) => { }
                WingResponse::NodeDef(def) => {
                    if def.parent_id == *parent {
                        ret2.push(def);
                    }
                }
                WingResponse::RequestEnd => {
                    break;
                }
            }
        }
        ret.push(ret2);
    }
    ret
}

fn add(cnt: usize, wing: &mut WingConsole, json_file: &mut File, raw: &mut Vec<u8>, parent_fullname: &str, nodes: &[WingNodeDef], ignore: bool) -> usize {
    let mut cnt = cnt;
    if !ignore {
        if let Some(mdl_def) = nodes.iter().find(|x| &x.name == "mdl" && x.node_type == libwing::NodeType::StringEnum) {
            let children = get_node_def(wing, nodes.iter().map(|x| x.id).collect());
            cnt += children.len();

            for (i, def) in nodes.iter().enumerate() {
                if def.index != 0 { continue; }
                let fullname =
                    if def.name.is_empty() {
                        String::new() + parent_fullname + "/" + &def.index.to_string()[..]
                    } else {
                        String::new() + parent_fullname + "/" + &def.name[..]
                    };

                let mut json = def.to_json();
                json.insert("fullname", fullname.clone()).unwrap();
                // println!("{}", jzon::stringify(json.clone()));
                writeln!(json_file, "{}", jzon::stringify(json)).unwrap();
                raw.push(0);
                for b in (fullname.len() as u16).to_be_bytes() { raw.push(b); }
                for b in fullname.clone().into_bytes() { raw.push(b); }
                for b in (def.raw.len() as u16).to_be_bytes() { raw.push(b); }
                raw.append(&mut def.raw.clone());

                cnt = add(cnt, wing, json_file, raw, &fullname, &children[i], false);
            }

            for item in mdl_def.string_enum.as_ref().unwrap().iter() {
                let parent_fullname = String::new() + parent_fullname + "/" + &item.item;
                wing.set_string(mdl_def.id, &item.item).unwrap();
                let children = get_node_def(wing, Vec::from([mdl_def.parent_id]));
                cnt = add(cnt, wing, json_file, raw, &parent_fullname, &children[0], true);
            }

            return cnt;
        }
    }

    let children = get_node_def(wing, nodes.iter().map(|x| x.id).collect());
    cnt += children.len();

    for (i, def) in nodes.iter().enumerate() {
        // skip mdl and other nodes since we handled them above
        if !ignore || def.index != 0 {
            let fullname =
                if def.name.is_empty() {
                    String::new() + parent_fullname + "/" + &def.index.to_string()[..]
                } else {
                    String::new() + parent_fullname + "/" + &def.name[..]
                };

            let mut json = def.to_json();
            json.insert("fullname", fullname.clone()).unwrap();
            // println!("{}", jzon::stringify(json.clone()));
            writeln!(json_file, "{}", jzon::stringify(json)).unwrap();
            raw.push(if ignore { def.index as u8 } else { 0 });
            for b in (fullname.len() as u16).to_be_bytes() { raw.push(b); }
            for b in fullname.clone().into_bytes() { raw.push(b); }
            for b in (def.raw.len() as u16).to_be_bytes() { raw.push(b); }
            raw.append(&mut def.raw.clone());

            cnt = add(cnt, wing, json_file, raw, &fullname, &children[i], false);
        }
    }
    print!("\rReceived {} nodes", cnt);
    cnt
}
fn main() -> Result<(),libwing::Error> {
    let mut args = Args::new(r#"
Usage: wingschema [-h host]

   -h host : IP address or hostname of Wing mixer. Default is to discover and connect to the first mixer found.
"#);
    let mut host = None;
    if args.has_next() && args.next() == "-h" { host = Some(args.next()); }
    let mut wing = WingConsole::connect(host.as_deref())?;

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
    writeln!(rust_file, "use crate::node::WingNodeDef;").unwrap();
    writeln!(rust_file, "lazy_static::lazy_static! {{").unwrap();
    let mut raw = Vec::<u8>::new();
    let children = get_node_def(&mut wing, Vec::from([0]));
    add(1, &mut wing, &mut json_file, &mut raw, "", &children[0], false);
    print!("\nFinishing up... ");
    std::io::stdout().flush().unwrap();

    writeln!(rust_file, "    pub static ref NAME_TO_DEF: HashMap<String, WingNodeDef> = {{").unwrap();
    writeln!(rust_file, "        let mut m = HashMap::new();").unwrap();
    write!(  rust_file, "        let d = b\"").unwrap();
    for b in raw { write!(rust_file, "\\x{:02X}", b).unwrap(); }
    writeln!(rust_file, "\";").unwrap();
    writeln!(rust_file, "        let mut i = 0;").unwrap();
    writeln!(rust_file, "        while i < d.len() {{").unwrap();
    writeln!(rust_file, "            let _is_fake = d[i];").unwrap();
    writeln!(rust_file, "            i += 1;").unwrap();
    writeln!(rust_file, "            let namelen = u16::from_be_bytes([d[i], d[i + 1]]) as usize;").unwrap();
    writeln!(rust_file, "            i += 2;").unwrap();
    writeln!(rust_file, "            let name = String::from_utf8(d[i..i + namelen].to_vec()).unwrap();").unwrap();
    writeln!(rust_file, "            i += namelen;").unwrap();
    writeln!(rust_file, "            let deflen = u16::from_be_bytes([d[i], d[i + 1]]) as usize;").unwrap();
    writeln!(rust_file, "            i += 2;").unwrap();
    writeln!(rust_file, "            let def = WingNodeDef::from_bytes(&d[i..i + deflen]);").unwrap();
    writeln!(rust_file, "            i += deflen;").unwrap();
    writeln!(rust_file, "            m.insert(name, def);").unwrap();
    writeln!(rust_file, "        }}").unwrap();
    writeln!(rust_file, "        m").unwrap();
    writeln!(rust_file, "    }};").unwrap();
    writeln!(rust_file, "}}").unwrap();


    println!("done");
    Ok(())
}
