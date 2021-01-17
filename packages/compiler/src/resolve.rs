// Copyright 2020-2021 postUI Lab. All rights reserved. MIT license.

use serde::Serialize;
use swc_ecmascript::parser::JscTarget;

pub type Target = JscTarget;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyDescriptor {
  /// The text specifier associated with the import/export statement.
  pub specifier: String,
  /// A flag indicating if the import is dynamic or not.
  pub is_dynamic: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CSSTemplate {
  pub quasis: Vec<String>,
  pub exprs: Vec<String>,
}

/// A Resolver to resolve aleph.js import/export URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolver {
  /// current component specifier
  pub specifier: String,
  /// dom helper module
  pub runtime_module: String,
  /// dependency graph
  pub dep_graph: Vec<DependencyDescriptor>,
  /// inline styles
  pub css: Option<CSSTemplate>,
}

impl Resolver {
  pub fn new(specifier: &str, runtime_module: &str) -> Self {
    Resolver {
      specifier: specifier.into(),
      runtime_module: runtime_module.into(),
      dep_graph: Vec::new(),
      css: None,
    }
  }
}

impl Default for Resolver {
  fn default() -> Self {
    Resolver {
      specifier: "./App.alef".into(),
      runtime_module: "alef-dom".into(),
      dep_graph: Vec::new(),
      css: None,
    }
  }
}

pub fn to_component_name(s: &str) -> String {
  let mut should_uppercase = true;
  let mut char_vec: Vec<char> = vec![];
  for c in s.trim_end_matches(".alef").chars() {
    if c >= 'a' && c <= 'z' {
      if should_uppercase {
        should_uppercase = false;
        char_vec.push(c.to_ascii_uppercase());
      } else {
        char_vec.push(c);
      }
    } else if c >= 'A' && c <= 'Z' {
      should_uppercase = false;
      char_vec.push(c);
    } else if (c >= '0' && c <= '9') && char_vec.len() > 0 {
      should_uppercase = false;
      char_vec.push(c);
    } else {
      should_uppercase = true
    }
  }
  if char_vec.len() == 0 {
    return "App".into();
  }
  char_vec.into_iter().collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_to_component_name() {
    assert_eq!(to_component_name("app.alef"), "App");
    assert_eq!(to_component_name("APP.alef"), "APP");
    assert_eq!(to_component_name("hello-world.alef"), "HelloWorld");
    assert_eq!(to_component_name("hello world.alef"), "HelloWorld");
    assert_eq!(to_component_name("hi798.alef"), "Hi798");
    assert_eq!(to_component_name("798hi.alef"), "Hi");
    assert_eq!(to_component_name("798.alef"), "App");
    assert_eq!(to_component_name("Hello 世界!.alef"), "Hello");
  }
}
