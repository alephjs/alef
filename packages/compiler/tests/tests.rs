mod common;

use common::{t, t_custom_runtime_module};
use regex::Regex;

#[test]
fn test_custom_runmtime_module() {
  let (code, _) = t_custom_runtime_module("./App.alef", "", "https://deno.land/x/alef/dom.ts");
  assert!(code.contains(" from \"https://deno.land/x/alef/dom.ts\";"));
  let (code, _) = t_custom_runtime_module("./App.alef", "", "window.__ALEF_DOM");
  assert!(code.contains("} = window.__ALEF_DOM;"));
}

#[test]
fn test_component_export() {
  let source = r#"
    let name: string = 'World'

    $t: <p>Hello {name}!</p>
  "#;
  let (code, _) = t("App.alef", source);
  assert!(code.contains("import { Component, Element, Memo } from \"alef-dom\";"));
  assert!(code.contains("export default class App extends Component"));
  assert!(code.contains("constructor(props)"));
  assert!(code.contains("super(props)"));
  assert!(code.contains("this.register(nodes)"));
}

#[test]
fn test_component_jsx() {
  let source = r#"
    let name: string = 'World'

    $t: <p>Hello <strong>{name}</strong>!</p>
  "#;
  let (code, _) = t("App.alef", source);
  assert!(code.contains("import { Component, Element, Memo } from \"alef-dom\";"));
  assert!(code.contains("Element(\"p\", null, \"Hello \", Element(\"strong\", null"));

  let r = Regex::new(r"Memo\(\(\)\s*=>\s*name\s*,\s*\[\s*0\s*\]\)").unwrap();
  assert!(r.is_match(code.as_str()));
}

#[test]
fn test_component_dirty() {
  let source = r#"
    let n = 0

    function increase() {
      n++
    }

    $t: <button onClick={e => { n-- }}>-</button>
    $t: <button onClick={increase}>+</button>
  "#;
  let (code, _) = t("App.alef", source);
  let r1 =
    Regex::new(r"onClick:\s*Dirty\(\(e\)\s*=>\s*\{\s*n--;?\s*\}\s*,\s*\[\s*0\s*\]\s*\)").unwrap();
  let r2 = Regex::new(
    r"const increase\s*=\s*Dirty\(function increase\(\)\s*\{\s*n\+\+;?\s*\},\s*\[\s*0\s*\]\s*\)",
  )
  .unwrap();
  assert!(r1.is_match(code.as_str()));
  assert!(r2.is_match(code.as_str()));
}
