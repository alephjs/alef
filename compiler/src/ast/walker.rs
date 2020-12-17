// Copyright 2020 the The Alef Component authors. All rights reserved. MIT license.

use super::{css::CSS, statement::*, AST};
use crate::resolve::Resolver;
use std::{cell::RefCell, rc::Rc};
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_fold_type, Fold};

/// AST walker for Alef Component.
pub struct ASTWalker {
  pub resolver: Rc<RefCell<Resolver>>,
}

impl ASTWalker {
  /// transform `swc_ecma_ast::Stmt` to `Vec<Statement>`
  fn transform_stmt(&self, stmt: &Stmt) -> Vec<Statement> {
    let mut stmts: Vec<Statement> = vec![];

    match stmt {
      Stmt::Decl(Decl::Var(VarDecl { kind, decls, .. })) => match kind {
        VarDeclKind::Const => {
          for decl in decls {
            let mut typed = ConstTyped::Regular;
            match decl.name {
              Pat::Ident(Ident { ref type_ann, .. })
              | Pat::Array(ArrayPat { ref type_ann, .. })
              | Pat::Object(ObjectPat { ref type_ann, .. }) => match type_ann {
                Some(TsTypeAnn { type_ann, .. }) => match type_ann.as_ref() {
                  TsType::TsTypeRef(TsTypeRef {
                    type_name: TsEntityName::Ident(Ident { sym, .. }),
                    type_params,
                    ..
                  }) => match sym.as_ref() {
                    "Prop" => {
                      typed = ConstTyped::Prop;
                      match type_params {
                        Some(type_params) => {
                          if type_params.params.len() == 1 {
                            match type_params.params.first() {
                              Some(param) => match param.as_ref() {
                                TsType::TsTypeRef(TsTypeRef {
                                  type_name: TsEntityName::Ident(Ident { sym, .. }),
                                  type_params: None,
                                  ..
                                }) => {
                                  if sym.eq("Slots") {
                                    typed = ConstTyped::Slots
                                  }
                                }
                                _ => {}
                              },
                              _ => {}
                            }
                          }
                        }
                        _ => {}
                      }
                    }
                    "Context" => typed = ConstTyped::Context,
                    "Memo" => typed = ConstTyped::Memo,
                    "FC" => match &decl.init {
                      Some(init) => match init.as_ref() {
                        Expr::Arrow(ArrowExpr { body, .. }) => match body {
                          BlockStmtOrExpr::BlockStmt(block_stmt) => {
                            let mut fc_stmts: Vec<Statement> = vec![];
                            for stmt in &block_stmt.stmts {
                              fc_stmts = [fc_stmts, self.transform_stmt(stmt)].concat()
                            }
                            stmts.push(Statement::FC(FCStatement { statements: fc_stmts }));
                            continue;
                          }
                          BlockStmtOrExpr::Expr(expr) => {
                            stmts.push(Statement::FC(FCStatement {
                              statements: self.transform_stmt(&Stmt::Expr(ExprStmt {
                                span: DUMMY_SP,
                                expr: expr.clone(),
                              })),
                            }));
                            continue;
                          }
                        },
                        Expr::Fn(FnExpr {
                          function:
                            Function {
                              body: Some(body),
                              is_generator: false,
                              ..
                            },
                          ..
                        }) => {
                          let mut fc_stmts: Vec<Statement> = vec![];
                          for stmt in &body.stmts {
                            fc_stmts = [fc_stmts, self.transform_stmt(stmt)].concat()
                          }
                          stmts.push(Statement::FC(FCStatement { statements: fc_stmts }));
                          continue;
                        }
                        _ => {}
                      },
                      _ => {}
                    },
                    _ => {}
                  },
                  _ => {}
                },
                _ => {}
              },
              _ => {}
            };
            stmts.push(Statement::Const(ConstStatement {
              typed,
              name: decl.name.clone(),
              expr: decl.init.clone().unwrap(),
            }))
          }
        }
        _ => {
          for decl in decls {
            let mut is_array = false;
            let mut is_ref = false;
            let is_async = match decl.init {
              Some(ref expr) => match expr.as_ref() {
                Expr::Await(_) => true,
                _ => false,
              },
              _ => false,
            };
            match decl.name {
              Pat::Ident(Ident { ref type_ann, .. })
              | Pat::Array(ArrayPat { ref type_ann, .. })
              | Pat::Object(ObjectPat { ref type_ann, .. }) => match type_ann {
                Some(TsTypeAnn { type_ann, .. }) => match type_ann.as_ref() {
                  TsType::TsArrayType(_) => is_array = true,
                  TsType::TsTypeRef(TsTypeRef {
                    type_name: TsEntityName::Ident(Ident { sym, .. }),
                    ..
                  }) => is_ref = sym.eq("Ref"),
                  _ => {}
                },
                _ => {}
              },
              _ => {}
            };
            stmts.push(Statement::Var(VarStatement {
              name: decl.name.clone(),
              expr: decl.init.clone(),
              is_array,
              is_ref,
              is_async,
            }))
          }
        }
      },
      Stmt::Labeled(labeled) => match labeled.label.as_ref() {
        "$" => stmts.push(Statement::SideEffect(SideEffectStatement {
          name: None,
          stmt: labeled.body.clone(),
        })),
        "$t" => match labeled.body.as_ref() {
          Stmt::Expr(ExprStmt { expr, .. }) => match expr.as_ref() {
            Expr::JSXElement(el) => stmts.push(Statement::Template(TemplateStatement::Element(
              el.as_ref().clone(),
            ))),
            Expr::JSXFragment(fragment) => stmts.push(Statement::Template(
              TemplateStatement::Fragment(fragment.clone()),
            )),
            _ => stmts.push(Statement::Stmt(Stmt::Labeled(labeled.clone()))),
          },
          _ => stmts.push(Statement::Stmt(Stmt::Labeled(labeled.clone()))),
        },
        "$style" => match labeled.body.as_ref() {
          Stmt::Expr(ExprStmt { expr, .. }) => match expr.as_ref() {
            Expr::Tpl(tpl) => stmts.push(Statement::Style(StyleStatement {
              css: Box::new(CSS::parse(tpl)),
            })),
            _ => stmts.push(Statement::Stmt(Stmt::Labeled(labeled.clone()))),
          },
          _ => stmts.push(Statement::Stmt(Stmt::Labeled(labeled.clone()))),
        },
        _ => {
          let label = labeled.label.as_ref();
          if label.starts_with("$_") {
            stmts.push(Statement::SideEffect(SideEffectStatement {
              name: Some(label.trim_start_matches("$_").into()),
              stmt: labeled.body.clone(),
            }))
          } else {
            stmts.push(Statement::Stmt(Stmt::Labeled(labeled.clone())))
          }
        }
      },
      _ => stmts.push(Statement::Stmt(stmt.clone())),
    };

    stmts
  }
}

/// Folds the component to an AST then stores it in resolver,
/// and returns a empty module.
impl Fold for ASTWalker {
  noop_fold_type!();

  fn fold_module_items(&mut self, module_items: Vec<ModuleItem>) -> Vec<ModuleItem> {
    let mut resolver = self.resolver.borrow_mut();
    let mut stmts: Vec<Statement> = vec![];

    for item in module_items {
      match item {
        ModuleItem::ModuleDecl(decl) => match decl {
          ModuleDecl::Import(ImportDecl {
            specifiers, src, ..
          }) => stmts.push(Statement::Import(ImportStatement {
            specifiers,
            src: src.value.as_ref().into(),
            is_alef_component: src.value.as_ref().ends_with(".alef"),
          })),
          ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { expr, .. }) => {
            stmts.push(Statement::Export(ExportStatement { expr }))
          }
          _ => {}
        },
        ModuleItem::Stmt(ref stmt) => stmts = [stmts, self.transform_stmt(stmt)].concat(),
      }
    }

    // store the AST to resolver
    resolver.ast = Some(AST { statements: stmts });

    // return a empty moudle
    vec![]
  }
}
