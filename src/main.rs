use egg::*;
use serde::ser::SerializeMap;
use std::fmt::Display;
use std::path;
use under::serialize;
use under::session;

use under::ao;

fn main() {
    /*
    let mut eg: EGraph<SymbolLang, ()> = Default::default();
    serialize::get_simple_egraph(&mut eg);
    eg.dot().to_svg("foo.svg").unwrap();
    let out = serialize::egraph_to_serialized_egraph(&mut eg);
    out.to_json_file("out.txt");
    serialize::egraph_to_and_or(&eg, String::from("test"));
    */

    //session::Session::new().go();
    ao::main()
}
