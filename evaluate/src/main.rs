use getopts;
use sxd_document;

use std::env;
use std::fs::File;
use std::io::{self, Read};

use sxd_document::parser::parse;

use xpath_rs::{Context, Factory, Value, XPath};

use getopts::Options;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options] FILE...", program);
    print!("{}", opts.usage(&brief));
}

fn build_xpath(xpath_str: &str) -> XPath {
    let factory = Factory::new();

    factory
        .build(xpath_str)
        .unwrap_or_else(|e| panic!("Unable to compile XPath {}: {}", xpath_str, e))
}

fn load_xml<R>(input: R) -> sxd_document::Package
where
    R: Read,
{
    let mut input = input;
    let mut data = String::new();

    if let Err(x) = input.read_to_string(&mut data) {
        panic!("Can't read: {}", x);
    }

    match parse(&data) {
        Ok(d) => d,
        Err(e) => {
            panic!("Unable to parse input XML: {}", e);
        }
    }
}

fn argument_name_value(s: &str, delim: char) -> Option<(&str, &str)> {
    let parts: Vec<_> = s.splitn(2, delim).collect();
    if parts.len() < 2 {
        None
    } else {
        Some((parts[0], parts[1]))
    }
}

fn build_variables(arguments: &getopts::Matches, context: &mut Context<'_>) {
    for boolean in arguments.opt_strs("boolean") {
        match argument_name_value(&boolean, '=') {
            Some((name, "true")) => context.set_variable(name, true),
            Some((name, "false")) => context.set_variable(name, false),
            Some((_, val)) => panic!("Unknown boolean value '{}'", val),
            None => panic!("boolean argument '{}' is malformed", boolean),
        }
    }

    for number in arguments.opt_strs("number") {
        match argument_name_value(&number, '=') {
            Some((name, val)) => match val.parse() {
                Ok(val) => context.set_variable(name, Value::Number(val)),
                Err(e) => panic!("Unknown numeric value '{}': {}", val, e),
            },
            None => panic!("number argument '{}' is malformed", number),
        }
    }

    for string in arguments.opt_strs("string") {
        match argument_name_value(&string, '=') {
            Some((name, val)) => context.set_variable(name, val),
            None => panic!("string argument '{}' is malformed", string),
        }
    }
}

fn build_namespaces(arguments: &getopts::Matches, context: &mut Context<'_>) {
    for ns_defn in arguments.opt_strs("namespace") {
        match argument_name_value(&ns_defn, ':') {
            Some((prefix, uri)) => context.set_namespace(prefix, uri),
            None => panic!("Namespace definition '{}' is malformed", ns_defn),
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let program_name = &args[0];

    let mut opts = Options::new();

    opts.reqopt("", "xpath", "The XPath to execute", "XPATH");
    opts.optmulti("", "namespace", "set namespace prefix", "PREFIX:URI");
    opts.optmulti("", "string", "set string variable", "NAME=VALUE");
    opts.optmulti("", "number", "set number variable", "NAME=VALUE");
    opts.optmulti("", "boolean", "set boolean variable", "NAME=VALUE");

    let arguments = match opts.parse(&args[1..]) {
        Ok(x) => x,
        Err(x) => {
            println!("{}", x);
            print_usage(program_name, &opts);
            return;
        }
    };

    let xpath_str = arguments.opt_str("xpath").expect("No XPath provided");
    let xpath = build_xpath(&xpath_str);

    for filename in &arguments.free {
        let package = if *filename == "-" {
            load_xml(io::stdin())
        } else {
            let file = File::open(filename).expect("Could not open XML file");
            load_xml(file)
        };

        let doc = package.as_document();

        let mut context = Context::new();
        build_variables(&arguments, &mut context);
        build_namespaces(&arguments, &mut context);

        let res = xpath.evaluate(&context, doc.root());

        println!("{:?}", res);
    }
}
