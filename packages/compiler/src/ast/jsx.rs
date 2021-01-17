// Copyright 2020-2021 postUI Lab. All rights reserved. MIT license.

use super::identmap::IdentMap;
use crate::resolve::Resolver;
use regex::Regex;
use std::{cell::RefCell, iter, mem, rc::Rc};
use swc_common::{iter::IdentifyLast, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_utils::{member_expr, quote_ident, ExprFactory, HANDLER};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JSXTransformer {
    pub resolver: Rc<RefCell<Resolver>>,
    pub scope_idents: Rc<RefCell<IdentMap>>,
}

impl JSXTransformer {
    fn create_ident(&self, name: &str) -> Ident {
        let mut scope_idents = self.scope_idents.borrow_mut();
        scope_idents.create_ident(name)
    }

    pub fn transform_element(&self, el: JSXElement) -> Expr {
        let element_ident = self.create_ident("Element");
        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: ExprOrSuper::Expr(Box::new(Expr::Ident(element_ident))),
            args: iter::once(jsx_name(el.opening.name).as_arg())
                .chain(iter::once(self.transform_attrs(el.opening.attrs).as_arg()))
                .chain({
                    el.children
                        .into_iter()
                        .filter_map(|c| self.transform_child(c))
                })
                .collect(),
            type_args: Default::default(),
        })
    }

    pub fn transform_fragment(&self, frag: JSXFragment) -> Expr {
        let frag_ident = self.create_ident("Fragment");
        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: ExprOrSuper::Expr(Box::new(Expr::Ident(frag_ident))),
            args: frag
                .children
                .into_iter()
                .filter_map(|c| self.transform_child(c))
                .collect(),
            type_args: Default::default(),
        })
    }

    pub fn transform_condition(&self, if_stmt: IfStmt) -> Stmt {
        Stmt::Empty(EmptyStmt { span: DUMMY_SP })
    }

    fn transform_child(&self, c: JSXElementChild) -> Option<ExprOrSpread> {
        Some(match c {
            JSXElementChild::JSXText(text) => {
                let s = Str {
                    span: text.span,
                    has_escape: text.raw != text.value,
                    value: jsx_text_to_string(text.value.as_ref()).into(),
                    kind: Default::default(),
                };
                if s.value.is_empty() {
                    return None;
                }
                Lit::Str(s).as_arg()
            }
            JSXElementChild::JSXElement(el) => self.transform_element(*el).as_arg(),
            JSXElementChild::JSXFragment(el) => self.transform_fragment(el).as_arg(),
            JSXElementChild::JSXExprContainer(JSXExprContainer { expr, .. }) => match expr {
                JSXExpr::Expr(e) => self.transform_expr(*e, false).as_arg(),
                JSXExpr::JSXEmptyExpr(..) => return None,
            },
            JSXElementChild::JSXSpreadChild(JSXSpreadChild { .. }) => {
                unimplemented!("jsx sperad child")
            }
        })
    }

    fn transform_attrs(&self, attrs: Vec<JSXAttrOrSpread>) -> Box<Expr> {
        if attrs.is_empty() {
            return Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })));
        }
        let is_complex = attrs.iter().any(|a| match *a {
            JSXAttrOrSpread::SpreadElement(..) => true,
            _ => false,
        });
        if is_complex {
            let mut args = vec![];
            let mut cur_obj_props = vec![];
            macro_rules! check {
                () => {{
                    if args.is_empty() || !cur_obj_props.is_empty() {
                        args.push(
                            ObjectLit {
                                span: DUMMY_SP,
                                props: mem::replace(&mut cur_obj_props, vec![]),
                            }
                            .as_arg(),
                        )
                    }
                }};
            }
            for attr in attrs {
                match attr {
                    JSXAttrOrSpread::JSXAttr(a) => {
                        cur_obj_props.push(PropOrSpread::Prop(Box::new(attr_to_prop(a))))
                    }
                    JSXAttrOrSpread::SpreadElement(e) => {
                        check!();
                        args.push(e.expr.as_arg());
                    }
                }
            }
            check!();
            Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: { member_expr!(DUMMY_SP, Object.assign).as_callee() },
                args,
                type_args: None,
            }))
        } else {
            Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: attrs
                    .into_iter()
                    .map(|a| match a {
                        JSXAttrOrSpread::JSXAttr(a) => a,
                        _ => unreachable!(),
                    })
                    .map(attr_to_prop)
                    .map(|v| match v {
                        Prop::KeyValue(KeyValueProp { key, value }) => {
                            Prop::KeyValue(KeyValueProp {
                                key: key.clone(),
                                value: Box::new(self.transform_expr(*value, is_event_prop(key))),
                            })
                        }
                        _ => v,
                    })
                    .map(Box::new)
                    .map(PropOrSpread::Prop)
                    .collect(),
            }))
        }
    }

    fn transform_expr(&self, expr: Expr, is_event: bool) -> Expr {
        let mut deps: Vec<usize> = vec![];
        let expr = if is_event {
            self.convert_dirty_expr(expr, &mut deps)
        } else {
            self.convert_memo_expr(expr, &mut deps)
        };
        if deps.len() > 0 {
            let call_ident = self.create_ident(if is_event { "Dirty" } else { "Memo" });
            return Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: ExprOrSuper::Expr(Box::new(Expr::Ident(call_ident))),
                args: iter::once(if is_event {
                    expr.as_arg()
                } else {
                    expr_to_arrow(expr).as_arg()
                })
                .chain(iter::once(
                    Expr::Array(ArrayLit {
                        span: DUMMY_SP,
                        elems: deps
                            .into_iter()
                            .map(|dep| {
                                Some(ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Lit(Lit::Num(Number {
                                        span: DUMMY_SP,
                                        value: dep as f64,
                                    }))),
                                })
                            })
                            .collect(),
                    })
                    .as_arg(),
                ))
                .collect(),
                type_args: Default::default(),
            });
        }
        expr
    }

    fn convert_memo_expr(&self, expr: Expr, deps: &mut Vec<usize>) -> Expr {
        let scope_idents = self.scope_idents.borrow();
        match expr {
            Expr::JSXElement(el) => self.transform_element(*el),
            Expr::JSXFragment(frag) => self.transform_fragment(frag),
            _ => scope_idents.convert_memo_expr(expr, deps),
        }
    }

    fn convert_dirty_expr(&self, expr: Expr, deps: &mut Vec<usize>) -> Expr {
        let scope_idents = self.scope_idents.borrow();
        scope_idents.convert_dirty_expr(expr, deps)
    }
}

fn jsx_name(name: JSXElementName) -> Box<Expr> {
    match name {
        JSXElementName::Ident(i) => {
            if i.sym.eq("this") {
                Box::new(Expr::This(ThisExpr { span: DUMMY_SP }))
            } else if i.sym.chars().next().unwrap().is_ascii_lowercase() {
                Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: i.sym,
                    has_escape: false,
                    kind: Default::default(),
                })))
            } else {
                Box::new(Expr::Ident(i))
            }
        }
        JSXElementName::JSXNamespacedName(JSXNamespacedName { name, .. }) => {
            HANDLER.with(|handler| {
                handler
                    .struct_span_err(
                        name.span,
                        "Alep Component does not support JSX Namespace yet.",
                    )
                    .emit()
            });
            Box::new(Expr::Invalid(Invalid { span: DUMMY_SP }))
        }
        JSXElementName::JSXMemberExpr(JSXMemberExpr { obj, prop }) => {
            Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: convert_jsx_obj(obj),
                prop: Box::new(Expr::Ident(prop)),
                computed: false,
            }))
        }
    }
}

fn convert_jsx_obj(obj: JSXObject) -> ExprOrSuper {
    match obj {
        JSXObject::Ident(i) => {
            if i.sym.eq("this") {
                return ExprOrSuper::Expr(Box::new(Expr::This(ThisExpr { span: DUMMY_SP })));
            }
            i.as_obj()
        }
        JSXObject::JSXMemberExpr(e) => {
            let e = *e;
            MemberExpr {
                span: DUMMY_SP,
                obj: convert_jsx_obj(e.obj),
                prop: Box::new(Expr::Ident(e.prop)),
                computed: false,
            }
            .as_obj()
        }
    }
}

fn attr_to_prop(a: JSXAttr) -> Prop {
    let key = to_prop_name(a.name);
    let value = a
        .value
        .map(|v| match v {
            JSXAttrValue::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::Expr(e),
                ..
            }) => e,
            JSXAttrValue::JSXElement(e) => Box::new(Expr::JSXElement(e)),
            JSXAttrValue::JSXFragment(e) => Box::new(Expr::JSXFragment(e)),
            JSXAttrValue::Lit(lit) => Box::new(lit.into()),
            JSXAttrValue::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::JSXEmptyExpr(_),
                ..
            }) => unreachable!("attr_to_prop(JSXEmptyExpr)"),
        })
        .unwrap_or_else(|| {
            Box::new(Expr::Lit(Lit::Bool(Bool {
                span: DUMMY_SP,
                value: true,
            })))
        });
    Prop::KeyValue(KeyValueProp { key, value })
}

fn to_prop_name(n: JSXAttrName) -> PropName {
    match n {
        JSXAttrName::Ident(i) => {
            if i.sym.contains('-') {
                PropName::Str(Str {
                    span: DUMMY_SP,
                    value: i.sym,
                    has_escape: false,
                    kind: StrKind::Normal {
                        contains_quote: false,
                    },
                })
            } else {
                PropName::Ident(i)
            }
        }
        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name }) => PropName::Str(Str {
            span: DUMMY_SP,
            value: format!("{}:{}", ns.sym, name.sym).into(),
            has_escape: false,
            kind: Default::default(),
        }),
    }
}

lazy_static! {
    static ref SPACE_NL_START: Regex = Regex::new("^\\s*\n\\s*").unwrap();
    static ref SPACE_NL_END: Regex = Regex::new("\\s*\n\\s*$").unwrap();
}

fn jsx_text_to_string(t: &str) -> String {
    if t.eq(" ") {
        return t.into();
    }
    if !t.contains(' ') && !t.contains('\n') {
        return t.into();
    }

    let s = SPACE_NL_START.replace_all(&t, "");
    let s = SPACE_NL_END.replace_all(&s, "");
    let need_leading_space = s.starts_with(' ');
    let need_trailing_space = s.ends_with(' ');

    let mut buf = String::new();

    if need_leading_space {
        buf.push(' ')
    }

    for (last, s) in s.split_ascii_whitespace().identify_last() {
        buf.push_str(s);
        if !last {
            buf.push(' ');
        }
    }

    if need_trailing_space && !buf.ends_with(' ') {
        buf.push(' ');
    }

    buf.into()
}

fn expr_to_arrow(expr: Expr) -> Expr {
    Expr::Arrow(ArrowExpr {
        span: DUMMY_SP,
        params: vec![],
        body: BlockStmtOrExpr::Expr(Box::new(expr)),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
    })
}

fn is_event_prop(name: PropName) -> bool {
    match name {
        PropName::Str(Str { value, .. }) => is_event_prop_name(value.as_ref()),
        PropName::Ident(Ident { sym, .. }) => is_event_prop_name(sym.as_ref()),
        _ => false,
    }
}

fn is_event_prop_name(name: &str) -> bool {
    let mut chars = name.chars();
    if let (Some(c0), Some(c1), Some(c2)) = (chars.next(), chars.next(), chars.next()) {
        return c0 == 'o' && c1 == 'n' && c2 >= 'A' && c2 <= 'Z';
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_str(value: &str) -> Str {
        Str {
            span: DUMMY_SP,
            value: value.into(),
            has_escape: false,
            kind: Default::default(),
        }
    }

    fn new_ident(id: &str) -> Ident {
        Ident {
            span: DUMMY_SP,
            sym: id.into(),
            type_ann: None,
            optional: false,
        }
    }

    #[test]
    fn test_is_event_prop() {
        assert_eq!(is_event_prop(PropName::Str(new_str("onClick"))), true);
        assert_eq!(is_event_prop(PropName::Str(new_str("onclick"))), false);
        assert_eq!(is_event_prop(PropName::Ident(new_ident("onClick"))), true);
        assert_eq!(is_event_prop(PropName::Ident(new_ident("onclick"))), false);
    }
}
