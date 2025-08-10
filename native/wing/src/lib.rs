use libwing::WingNodeData;
use rustler::{ ResourceArc,NifTaggedEnum};
use rustler::{Env, Term, NifResult, Encoder, OwnedEnv, LocalPid};

use libwing::{WingConsole, WingResponse,WingNodeDef,DiscoveryInfo};

use std::sync::Mutex;

rustler::atoms! {
    channel,
    mix,
    aux,
    main,
    matrix
}

struct ExWing { pub wing: Mutex<WingConsole> }

type WingArc = ResourceArc<ExWing>;

#[derive(NifTaggedEnum)]
pub enum WingResponseSimple {
    RequestEnd,
    NodeDef(WingNodeDef),
    NodeData(i32, WingNodeData),
    NodeDataSimple(String, WingNodeData),
}

fn on_load(env: Env, _info: Term) -> bool {
    let _ = rustler::resource!(ExWing, env);
    true
}

#[rustler::nif(schedule = "DirtyCpu")]
fn connect() -> WingArc {
    ResourceArc::new(
        ExWing {
            wing: Mutex::new(WingConsole::connect(None).unwrap()),
        }
    )
}

#[rustler::nif(schedule = "DirtyCpu")]
fn connect_with_host(host: Option<String>) -> WingArc {
    ResourceArc::new(
        ExWing {
            wing: Mutex::new(WingConsole::connect(host.as_deref()).unwrap()),
        }
    )
}

#[rustler::nif(schedule = "DirtyCpu")]
fn read_simple(wing_arc: WingArc) -> WingResponseSimple {
    let mut wing = wing_arc.wing.lock().unwrap();
    let wing: &mut WingConsole = &mut *wing;

    loop {
        if let Ok(WingResponse::NodeData(id, data)) =  wing.read() {
            match WingConsole::id_to_defs(id) {
                
                Some(defs) if defs.is_empty() => return WingResponseSimple::NodeData(id, data),
                Some(defs) if defs.len() == 1 => {
                    let longname = format!("{}", defs[0].0);
                    return WingResponseSimple::NodeDataSimple(longname, data);
                }
                Some(defs) if (defs.len() > 1) => continue,
                Some(_) => continue,
                None => continue,
            }
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn read(wing_arc: WingArc) -> WingResponse {
    let mut wing = wing_arc.wing.lock().unwrap();
    let wing: &mut WingConsole = &mut *wing;

    loop {
        if let Ok(response) = wing.read() {
            return response;
        }
    }
}
#[rustler::nif(schedule = "DirtyCpu")]
fn scan() -> Vec<DiscoveryInfo> {
    match WingConsole::scan(false) {
        Ok(discovery_info) => discovery_info,
        Err(_) => Vec::new(), 
    }
}

#[rustler::nif]
fn start_meter_thread(host: Option<String>, pid_term: Term, meters_term: Term) -> NifResult<()> {
    let pid: LocalPid = pid_term.decode()?;
    // Accept list of tuples: {atom, integer}
    let list: Vec<(rustler::types::atom::Atom, u8)> = meters_term.decode()?;
    let meters: Vec<libwing::Meter> = list.into_iter().map(|(kind, idx)| {
        if kind == channel() {
            libwing::Meter::Channel(idx)
        } else if kind == mix() {
            libwing::Meter::Bus(idx)
        } else if kind == aux() {
            libwing::Meter::Aux(idx)
        } else if kind == main() {
            libwing::Meter::Main(idx)
        } else if kind == matrix() {
            libwing::Meter::Matrix(idx)
        } else {
            libwing::Meter::Channel(1)
        }
    }).collect();
    std::thread::spawn(move || {
        let mut wing = match libwing::WingConsole::connect(host.as_deref()) {
            Ok(w) => w,
            Err(_) => return,
        };
        if wing.request_meter(&meters).is_err() {
            return;
        }
        loop {
            if let Ok((_id, values)) = wing.read_meters() {
                let msg: Vec<i32> = values.iter().map(|v| *v as i32).collect();
                let mut env = OwnedEnv::new();
                let _ = env.send_and_clear(&pid, |env| (rustler::types::atom::ok(), msg).encode(env));
            }
        }
    });
    Ok(())
}

#[rustler::nif]
fn start_property_thread(host: Option<String>, pid_term: Term, prop_id: i32) -> NifResult<()> {
    let pid: LocalPid = pid_term.decode()?;
    std::thread::spawn(move || {
        // Add a small delay to avoid overwhelming the console with simultaneous connections
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let mut wing = None;
        
        // Retry connection up to 3 times with increasing delays
        for attempt in 1..=3 {
            match libwing::WingConsole::connect(host.as_deref()) {
                Ok(w) => {
                    wing = Some(w);
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to connect for property thread {} (attempt {}): {:?}", prop_id, attempt, e);
                    if attempt < 3 {
                        std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
                    }
                }
            }
        }
        
        let mut wing = match wing {
            Some(w) => w,
            None => {
                eprintln!("Failed to connect for property thread {} after 3 attempts", prop_id);
                return;
            }
        };
        
        if let Err(e) = wing.request_node_data(prop_id) {
            eprintln!("Failed to request node data for property {}: {:?}", prop_id, e);
            return;
        }
        
        loop {
            match wing.read() {
                Ok(libwing::WingResponse::NodeData(id, data)) => {
                    if id == prop_id {
                        let mut env = OwnedEnv::new();
                        let float_value = data.get_float();
                        let msg = (
                            rustler::types::atom::ok(),
                            id,
                            float_value
                        );
                        let _ = env.send_and_clear(&pid, |env| msg.encode(env));
                    }
                }
                Ok(libwing::WingResponse::RequestEnd) => {
                    // Request end, continue reading
                    continue;
                }
                Ok(libwing::WingResponse::NodeDef(_)) => {
                    // Node definition, continue reading
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading from console for property {}: {:?}", prop_id, e);
                    // Try to reconnect after a delay
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    wing = match libwing::WingConsole::connect(host.as_deref()) {
                        Ok(mut w) => {
                            if let Err(_) = w.request_node_data(prop_id) {
                                return;
                            }
                            w
                        },
                        Err(_) => return,
                    };
                }
            }
        }
    });
    Ok(())
}

#[rustler::nif]
fn init_wing_thread(host: Option<String>) -> NifResult<(WingArc, rustler::types::atom::Atom)> {
    match WingConsole::connect(host.as_deref()) {
        Ok(wing_console) => {
            let wing_arc = ResourceArc::new(ExWing {
                wing: Mutex::new(wing_console),
            });
            Ok((wing_arc, rustler::types::atom::ok()))
        }
        Err(e) => {
            let error_msg = format!("Failed to connect: {:?}", e);
            Err(rustler::Error::Term(Box::new(error_msg)))
        }
    }
}

#[rustler::nif]
fn name_to_id(name: String) -> i32 {
    libwing::WingConsole::name_to_id(&name).unwrap_or(-1)
}

#[rustler::nif]
fn set_float(wing_arc: WingArc, id: i32, value: f32) -> Result<(), String> {
    let mut wing = wing_arc.wing.lock().unwrap();
    wing.set_float(id, value).map_err(|e| format!("{:?}", e))
}

#[rustler::nif]
fn request_node_data(wing_arc: WingArc, id: i32) -> Result<(), String> {
    let mut wing = wing_arc.wing.lock().unwrap();
    wing.request_node_data(id).map_err(|e| format!("{:?}", e))
}

rustler::init!(
    "Elixir.Wing",
    load = on_load
);

