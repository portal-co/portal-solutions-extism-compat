use std::{collections::BTreeMap, sync::{LazyLock, Mutex}};

use base64::Engine;
use extism_pdk::*;
#[link(wasm_import_module = "_ext")]
unsafe extern "C" {
    fn read_guest(id: u32, idx: u64) -> u8;
    fn write_guest(id: u32, idx: u64, value: u8);
    fn len_guest(id: u32) -> u64;
}

#[unsafe(export_name = "_px_app_core")]
extern "C" fn core_call() {}

pub static pollers: LazyLock<Mutex<BTreeMap<[u8; 32], ([u8; 32], String)>>> =
    LazyLock::new(|| Mutex::new(Default::default()));

pub fn do_login_poll(name: &str) -> Result<BTreeMap<[u8; 32], [u8; 32]>, Error> {
    let mut p = pollers.lock().unwrap();
    let mut m: BTreeMap<[u8; 32], [u8; 32]> = BTreeMap::new();
    let mut blob = format!("");
    let mut idxs = vec![];
    for (k, (v, s)) in p.iter() {
        if s != name {
            continue;
        }
        blob = format!(
            "{blob}${}",
            base64::engine::general_purpose::URL_SAFE.encode(k)
        );
        idxs.push(*k);
        // m.insert(*k)
    }
    let r = http::request::<Memory>(
        &HttpRequest {
            url: format!("{name}/login/auto/poll/b64/{blob}"),
            headers: Default::default(),
            method: Some(format!("GET")),
        },
        None,
    )?;
    for (c, k) in r.body().chunks(32).zip(idxs) {
        let c: [u8; 32] = c.try_into().unwrap();
        if c == [0u8; 32] {
            continue;
        }
        let Some((v, _)) = p.remove(&k) else {
            continue;
        };
        let v = std::array::from_fn(|i| c[i] ^ v[i]);
        m.insert(k, v);
    }
    Ok(m)
}
