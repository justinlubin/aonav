use under::ao;
use under::jgf;

use std::fs::File;
use std::io::Write;
use std::process::Command;

fn main() {
    let s = std::fs::read_to_string("examples/shared.json").unwrap();

    let d: jgf::Data = serde_json::from_str(&s).unwrap();

    let g = match d {
        jgf::Data::Single { graph } => graph,
        jgf::Data::Multi { .. } => panic!("multi not supported"),
    };

    let a: ao::AndOrGraph<String, String> = g.try_into().unwrap();

    let mut dot_file = File::create("out/out.dot").unwrap();
    write!(&mut dot_file, "{}", a.dot()).unwrap();

    let pdf_file = File::create("out/out.pdf").unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg("out/out.dot")
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();

    /*
    let mut eg: EGraph<SymbolLang, ()> = Default::default();
    serialize::get_simple_egraph(&mut eg);
    eg.dot().to_svg("foo.svg").unwrap();
    let out = serialize::egraph_to_serialized_egraph(&mut eg);
    out.to_json_file("out.txt");
    serialize::egraph_to_and_or(&eg, String::from("test"));
    */

    //session::Session::new().go();
}
