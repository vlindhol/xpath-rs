use super::{Function,EvaluationContext,Functions,Value};

struct Count;

impl Function for Count {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Value<'d>
    {
        // TODO: verify arguments
        let arg = &args[0];
        match arg {
            &Value::Nodes(ref nodeset) => Value::Number(nodeset.size() as f64),
            _ => panic!("Expected a nodeset"),
        }
    }
}

struct Not;

impl Function for Not {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Value<'d>
    {
        // TODO: verify arguments
        let arg = &args[0];
        match arg {
            &Value::Boolean(v) => Value::Boolean(!v),
            _ => panic!("Expected a boolean"),
        }
    }
}

struct True;

impl Function for True {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        _args: Vec<Value<'d>>) -> Value<'d>
    {
        Value::Boolean(true)
    }
}

struct False;

impl Function for False {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        _args: Vec<Value<'d>>) -> Value<'d>
    {
        Value::Boolean(false)
    }
}

pub fn register_core_functions(functions: &mut Functions) {
    functions.insert("count".to_string(), box Count);
    functions.insert("not".to_string(), box Not);
    functions.insert("true".to_string(), box True);
    functions.insert("false".to_string(), box False);
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use document::Package;
    use super::super::{EvaluationContext,Function,Value};
    use super::Count;

    #[test]
    fn count_counts_nodes_in_nodeset() {
        let package = Package::new();
        let doc = package.as_document();
        let nodeset = nodeset![doc.root()];

        let functions = HashMap::new();
        let variables = HashMap::new();
        let namespaces = HashMap::new();

        let context = EvaluationContext::new(doc.root(), &functions, &variables, &namespaces);
        let args = vec![Value::Nodes(nodeset)];
        let r = Count.evaluate(&context, args);

        assert_eq!(Value::Number(1.0), r);
    }
}
