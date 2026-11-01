use lesscode_core::{AstGrep, Python, Tsx, TypeScript};

#[test]
fn python_grammar_preserves_source_and_recognizes_function() {
    let source = "def greet(name):\r\n    return 'Hello, ' + name\r\n";
    let tree = AstGrep::new(source, Python);
    let root = tree.root();
    assert_eq!(root.kind(), "module");
    assert!(!root.get_inner_node().has_error());
    assert_eq!(root.text(), source);
    assert_eq!(root.child(0).unwrap().kind(), "function_definition");
}

#[test]
fn typescript_grammar_recognizes_types() {
    let source = "interface User { name: string }\nconst greet = (u: User): string => u.name;\n";
    let tree = AstGrep::new(source, TypeScript);
    let root = tree.root();
    assert_eq!(root.kind(), "program");
    assert!(!root.get_inner_node().has_error());
    assert_eq!(root.text(), source);
    assert_eq!(root.child(0).unwrap().kind(), "interface_declaration");
}

#[test]
fn tsx_grammar_recognizes_jsx_and_types() {
    let source = "const View = ({name}: {name: string}) => <h1>{name}</h1>;\n";
    let tree = AstGrep::new(source, Tsx);
    let root = tree.root();
    assert!(!root.get_inner_node().has_error());
    assert_eq!(root.text(), source);
    assert!(root.dfs().any(|node| node.kind() == "jsx_element"));
    let ts = AstGrep::new(source, TypeScript);
    assert!(ts.root().get_inner_node().has_error());
}

#[test]
fn malformed_inputs_remain_detectable() {
    assert!(AstGrep::new("def broken(:\n", Python)
        .root()
        .get_inner_node()
        .has_error());
    assert!(AstGrep::new("const value: = ;", TypeScript)
        .root()
        .get_inner_node()
        .has_error());
    assert!(AstGrep::new("const v = <div>{</div>;", Tsx)
        .root()
        .get_inner_node()
        .has_error());
}
