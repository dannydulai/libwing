use libwing::WingNodeData;
use rustler::{ ResourceArc,NifTaggedEnum};
use rustler::{Env, Term, NifResult, Encoder, OwnedEnv, LocalPid};

use libwing::{WingConsole, WingResponse,WingNodeDef,DiscoveryInfo};

use std::sync::Mutex;

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
    rustler::resource!(ExWing, env);
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
fn start_meter_thread(host: Option<String>, pid_term: Term) -> NifResult<()> {
    let pid: LocalPid = pid_term.decode()?;
    std::thread::spawn(move || {
        let meters: Vec<libwing::Meter> = (0..16).map(libwing::Meter::Channel).collect();
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
                env.send_and_clear(&pid, |env| (rustler::types::atom::ok(), msg).encode(env));
            }
        }
    });
    Ok(())
}

rustler::init!(
    "Elixir.Wing",
    [connect, read_simple, read, scan, start_meter_thread],
    load = on_load
);

