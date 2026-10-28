//! Offline QR generator for VectorCraft plug-in ABI v1.
use qrcodegen::{QrCode, QrCodeEcc};
use serde::Serialize;
use serde_json::{Value, json};

const MANIFEST: &str = r#"{
 "id":"nl.bram.vectorcraft.qrcode","name":"QR-code maken","version":"1.0.0","kind":"filter","budget":"heavy",
 "author":"Bram","description":"Voeg tekst of een URL toe als een bewerkbare vector-QR-code. Werkt volledig offline.",
 "params":{
  "tekst":{"type":"string","default":"https://example.com","maxLength":4096,"label":"Tekst of URL"},
  "formaat":{"type":"number","min":10,"max":1000,"default":30,"label":"Formaat incl. scanmarge","unit":"mm"},
  "foutcorrectie":{"type":"choice","options":["L (7%)","M (15%)","Q (25%)","H (30%)"],"default":"M (15%)","label":"Foutcorrectie"},
  "witteAchtergrond":{"type":"bool","default":true,"label":"Witte achtergrond"}
 }
}"#;

#[unsafe(no_mangle)]
pub extern "C" fn vc_abi_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn vc_manifest() -> u64 {
    pack(MANIFEST.as_bytes())
}
#[unsafe(no_mangle)]
pub extern "C" fn vc_alloc(size: u32) -> u32 {
    let mut bytes = Vec::<u8>::with_capacity(size.max(1) as usize);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr as u32
}
fn pack(b: &[u8]) -> u64 {
    ((b.len() as u64) << 32) | b.as_ptr() as u64
}
/// # Safety
/// The host provides valid allocated slices of input_len / params_len bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vc_run(
    input: *const u8,
    input_len: u32,
    params: *const u8,
    params_len: u32,
) -> i64 {
    let (input, params) = unsafe {
        (
            std::slice::from_raw_parts(input, input_len as usize),
            std::slice::from_raw_parts(params, params_len as usize),
        )
    };
    let (Ok(doc), Ok(p)) = (
        serde_json::from_slice::<Value>(input),
        serde_json::from_slice::<Value>(params),
    ) else {
        return -1;
    };
    let result = build(&doc, &p);
    let bytes = match result {
        Ok(out) => Ok(encode_output(&out)),
        Err(error) => serde_json::to_vec(&json!({"error":error})),
    };
    let Ok(bytes) = bytes else {
        return -2;
    };
    pack(bytes.leak()) as i64
}
#[derive(Serialize)]
struct Anchor {
    p: [f64; 2],
}
#[derive(Serialize)]
struct Subpath {
    closed: bool,
    anchors: [Anchor; 4],
}
#[derive(Serialize)]
struct Geometry {
    subpaths: Vec<Subpath>,
}
#[derive(Serialize)]
struct PathObject {
    #[serde(rename = "type")]
    kind: &'static str,
    name: String,
    path: Geometry,
    #[serde(rename = "fillRule")]
    fill_rule: &'static str,
    fills: Value,
    strokes: Vec<Value>,
    opacity: f64,
}
#[derive(Serialize)]
#[serde(untagged)]
enum Item {
    Existing { id: Value },
    New(PathObject),
}
#[derive(Serialize)]
struct Output {
    objects: Vec<Item>,
}
fn rect(x: f64, y: f64, w: f64, h: f64) -> Subpath {
    Subpath {
        closed: true,
        anchors: [
            Anchor { p: [x, y] },
            Anchor { p: [x + w, y] },
            Anchor { p: [x + w, y + h] },
            Anchor { p: [x, y + h] },
        ],
    }
}
fn object(name: &str, subpaths: Vec<Subpath>, k: f64) -> Item {
    Item::New(PathObject {
        kind: "path",
        name: name.into(),
        path: Geometry { subpaths },
        fill_rule: "nonzero",
        fills: json!([{"type":"solid","color":{"model":"gray","k":k}}]),
        strokes: vec![],
        opacity: 1.0,
    })
}
fn build(doc: &Value, p: &Value) -> Result<Output, String> {
    let text = p["tekst"].as_str().unwrap_or("");
    if text.trim().is_empty() {
        return Err("Vul tekst of een URL in.".into());
    }
    let ec = match p["foutcorrectie"]
        .as_str()
        .unwrap_or("M (15%)")
        .chars()
        .next()
    {
        Some('L') => QrCodeEcc::Low,
        Some('Q') => QrCodeEcc::Quartile,
        Some('H') => QrCodeEcc::High,
        _ => QrCodeEcc::Medium,
    };
    let Ok(code) = QrCode::encode_text(text, ec) else {
        return Err("Deze tekst is te lang voor een QR-code met deze foutcorrectie. Gebruik minder tekst of een lager niveau.".into());
    };
    let mm = p["formaat"].as_f64().unwrap_or(30.0);
    if !mm.is_finite() || !(10.0..=1000.0).contains(&mm) {
        return Err("Kies een formaat tussen 10 en 1000 mm.".into());
    }
    let size = mm * 72.0 / 25.4;
    let board = p["_context"]["artboard"].as_array();
    let (x, y) = if let Some(b) = board.filter(|b| b.len() == 4) {
        (
            (b[0].as_f64().unwrap_or(0.0) + b[2].as_f64().unwrap_or(612.0) - size) / 2.0,
            (b[1].as_f64().unwrap_or(0.0) + b[3].as_f64().unwrap_or(792.0) - size) / 2.0,
        )
    } else {
        (0.0, 0.0)
    };
    // Return only IDs for existing objects: the host keeps all of their properties.
    let mut objects: Vec<Item> = doc["objects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| o.get("id").map(|id| Item::Existing { id: id.clone() }))
        .collect();
    if p["witteAchtergrond"].as_bool().unwrap_or(true) {
        objects.push(object(
            "QR-code — witte scanmarge",
            vec![rect(x, y, size, size)],
            0.0,
        ));
    }
    let n = code.size() as usize;
    let unit = size / (n + 8) as f64; // Always reserve four light modules around the code.
    let mut paths = Vec::new();
    for row in 0..n {
        let mut col = 0;
        while col < n {
            if !code.get_module(col as i32, row as i32) {
                col += 1;
                continue;
            }
            let start = col;
            while col < n && code.get_module(col as i32, row as i32) {
                col += 1;
            }
            paths.push(rect(
                x + (start + 4) as f64 * unit,
                y + (row + 4) as f64 * unit,
                (col - start) as f64 * unit,
                unit,
            ));
        }
    }
    objects.push(object("QR-code", paths, 1.0));
    Ok(Output { objects })
}

// Fixed decimal coordinates avoid expensive general floating-point formatting in the interpreter.
// Precision is 0.0001 pt, far below output/printing precision.
fn encode_output(out: &Output) -> Vec<u8> {
    use std::fmt::Write;
    fn coord(s: &mut String, v: f64) {
        let n = (v * 10000.0).round() as i64;
        if n < 0 {
            s.push('-');
        }
        let n = n.unsigned_abs();
        write!(s, "{}.{:04}", n / 10000, n % 10000).unwrap();
    }
    let mut s = String::from("{\"objects\":[");
    for (i, item) in out.objects.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        match item {
            Item::Existing { id } => {
                write!(s, "{{\"id\":{}}}", id).unwrap();
            }
            Item::New(o) => {
                write!(s,"{{\"type\":\"path\",\"name\":{},\"fillRule\":\"nonzero\",\"fills\":{},\"strokes\":[],\"opacity\":1,\"path\":{{\"subpaths\":[",serde_json::to_string(&o.name).unwrap(),o.fills).unwrap();
                for (j, p) in o.path.subpaths.iter().enumerate() {
                    if j > 0 {
                        s.push(',');
                    }
                    s.push_str("{\"closed\":true,\"anchors\":[");
                    for (k, a) in p.anchors.iter().enumerate() {
                        if k > 0 {
                            s.push(',');
                        }
                        s.push_str("{\"p\":[");
                        coord(&mut s, a.p[0]);
                        s.push(',');
                        coord(&mut s, a.p[1]);
                        s.push_str("]}");
                    }
                    s.push_str("]}");
                }
                s.push_str("]}}");
            }
        }
    }
    s.push_str("]}");
    s.into_bytes()
}

#[cfg(test)]
fn generate(doc: &Value, p: &Value) -> Value {
    match build(doc, p) {
        Ok(out) => serde_json::to_value(out).unwrap(),
        Err(error) => json!({"error":error}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserve_selection_and_size() {
        let out = generate(
            &json!({"objects":[{"id":42,"path":{"custom":true}}]}),
            &json!({"tekst":"https://example.com","formaat":30,"_context":{"artboard":[100,200,700,1000]}}),
        );
        assert_eq!(out["objects"][0], json!({"id":42}));
        assert_eq!(out["objects"].as_array().unwrap().len(), 3);
        let anchors = out["objects"][1]["path"]["subpaths"][0]["anchors"]
            .as_array()
            .unwrap();
        let width = anchors[1]["p"][0].as_f64().unwrap() - anchors[0]["p"][0].as_f64().unwrap();
        assert!((width - 30.0 * 72.0 / 25.4).abs() < 1e-9);
        assert!((anchors[0]["p"][0].as_f64().unwrap() + width / 2.0 - 400.0).abs() < 1e-9);
    }
    #[test]
    fn invalid_and_overflow() {
        for text in ["".to_owned(), "  ".to_owned(), "x".repeat(4000)] {
            assert!(
                generate(&json!({"objects":[]}), &json!({"tekst":text}))
                    .get("error")
                    .is_some()
            );
        }
    }
    #[test]
    fn unicode_and_transparent() {
        let out = generate(
            &json!({"objects":[]}),
            &json!({"tekst":"Café — 日本語 🐈","witteAchtergrond":false}),
        );
        assert_eq!(out["objects"].as_array().unwrap().len(), 1);
        assert!(
            out["objects"][0]["path"]["subpaths"]
                .as_array()
                .unwrap()
                .len()
                > 20
        );
    }
}
