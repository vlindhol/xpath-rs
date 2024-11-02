use std::borrow::ToOwned;
use sxd_document::{dom, parser};
use xpath_rs::{context, function, nodeset};
use xpath_rs::{evaluate_xpath, Context, Factory, Value};

#[test]
fn functions_accept_arguments() {
    with_document("<a/>", |doc| {
        let result = evaluate_xpath(&doc, "concat('hello', ' ', 'world')");

        assert_eq!(Ok(Value::String("hello world".to_owned())), result);
    });
}

#[test]
fn functions_implicitly_coerce_argument_types() {
    with_document("<a>true</a>", |doc| {
        // We are searching a nodeset for a boolean. Both should be
        // converted to strings by `contains`.
        let result = evaluate_xpath(&doc, "count(//*[contains(., true)])");

        assert_eq!(Ok(Value::Number(1.0)), result);
    });
}

#[test]
fn axis_predicate_order() {
    with_document("<a><b><c/></b><b><c/></b></a>", |doc| {
        // All the `c` elements that are the first child
        let result = evaluate_xpath(&doc, "//c[1]");

        let a = doc.root().children()[0].element().expect("No element a");
        let b0 = a.children()[0].element().expect("No element b0");
        let b1 = a.children()[1].element().expect("No element b1");
        let c0 = b0.children()[0].element().expect("No element c0");
        let c1 = b1.children()[0].element().expect("No element c1");
        assert_eq!(Ok(Value::Nodeset(nodeset![c0, c1])), result);
    });
}

#[test]
fn position_function_in_predicate() {
    with_document("<a><b/><b/></a>", |doc| {
        let result = evaluate_xpath(&doc, "count(//a/*[position() = 2])");

        assert_eq!(Ok(Value::Number(1.0)), result);
    });
}

#[test]
fn variables_with_qualified_names() {
    with_document("<a/>", |doc| {
        let mut setup = Setup::new();
        setup.context.set_variable(("uri:namespace", "name"), 42.0);
        setup.context.set_namespace("prefix", "uri:namespace");

        let result = setup.evaluate(&doc, "$prefix:name");

        assert_eq!(42.0, result);
    });
}

#[test]
fn functions_with_qualified_names() {
    with_document("<a/>", |doc| {
        let mut setup = Setup::new();
        setup
            .context
            .set_function(("uri:namespace", "constant"), ConstantValueFunction(42.0));
        setup.context.set_namespace("prefix", "uri:namespace");

        let result = setup.evaluate(&doc, "prefix:constant()");

        assert_eq!(42.0, result);
    });
}

#[test]
fn nodesets_are_unique() {
    with_document("<a/>", |doc| {
        let result = evaluate_xpath(&doc, "/ | /");

        assert_eq!(Ok(Value::Nodeset(nodeset![doc.root()])), result);
    });
}

fn with_document<F>(xml: &str, f: F)
where
    F: FnOnce(dom::Document<'_>),
{
    let package = parser::parse(xml).expect("Unable to parse test XML");
    f(package.as_document());
}

#[derive(Default)]
struct Setup<'d> {
    context: Context<'d>,
    factory: Factory,
}

impl<'d> Setup<'d> {
    fn new() -> Setup<'d> {
        Default::default()
    }

    fn evaluate(&self, doc: &'d dom::Document<'d>, xpath: &str) -> Value<'d> {
        let xpath = self.factory.build(xpath).expect("Unable to build XPath");
        xpath
            .evaluate(&self.context, doc.root())
            .expect("Unable to evaluate XPath")
    }
}

struct ConstantValueFunction(f64);

impl function::Function for ConstantValueFunction {
    fn evaluate<'d>(
        &self,
        _context: &context::Evaluation<'_, 'd>,
        _args: Vec<Value<'d>>,
    ) -> Result<Value<'d>, function::Error> {
        Ok(Value::Number(self.0))
    }
}
