use biome_analyze::context::RuleContext;
use biome_analyze::{declare_rule, ActionCategory, Ast, FixKind, Rule, RuleDiagnostic};
use biome_console::markup;

use biome_diagnostics::Applicability;
use biome_js_syntax::{AnyTsType, TsTypeConstraintClause};
use biome_rowan::{AstNode, BatchMutationExt};

use crate::JsRuleAction;

declare_rule! {
    /// Disallow using `any` or `unknown` as type constraint.
    ///
    /// Generic type parameters (`<T>`) in TypeScript may be **constrained** with [`extends`](https://www.typescriptlang.org/docs/handbook/generics.html#generic-constraints).
    /// A supplied type must then be a subtype of the supplied constraint.
    /// All types are subtypes of `any` and `unknown`.
    /// It is thus useless to extend from `any` or `unknown`.
    ///
    /// Source: https://typescript-eslint.io/rules/no-unnecessary-type-constraint/
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```ts,expect_diagnostic
    /// interface FooAny<T extends any> {}
    /// ```
    /// ```ts,expect_diagnostic
    /// type BarAny<T extends any> = {};
    /// ```
    /// ```ts,expect_diagnostic
    /// class BazAny<T extends any> {
    /// }
    /// ```
    /// ```ts,expect_diagnostic
    /// class BazAny {
    ///   quxAny<U extends any>() {}
    /// }
    /// ```
    /// ```ts,expect_diagnostic
    /// const QuuxAny = <T extends any>() => {};
    /// ```
    /// ```ts,expect_diagnostic
    /// function QuuzAny<T extends any>() {}
    /// ```
    ///
    /// ```ts,expect_diagnostic
    /// interface FooUnknown<T extends unknown> {}
    /// ```
    /// ```ts,expect_diagnostic
    /// type BarUnknown<T extends unknown> = {};
    /// ```
    /// ```ts,expect_diagnostic
    /// class BazUnknown<T extends unknown> {
    /// }
    /// ```ts,expect_diagnostic
    /// class BazUnknown {
    ///   quxUnknown<U extends unknown>() {}
    /// }
    /// ```
    /// ```ts,expect_diagnostic
    /// const QuuxUnknown = <T extends unknown>() => {};
    /// ```
    /// ```ts,expect_diagnostic
    /// function QuuzUnknown<T extends unknown>() {}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```ts
    /// interface Foo<T> {}
    ///
    /// type Bar<T> = {};
    ///```
    pub(crate) NoUselessTypeConstraint {
        version: "1.0.0",
        name: "noUselessTypeConstraint",
        recommended: true,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoUselessTypeConstraint {
    type Query = Ast<TsTypeConstraintClause>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let node = ctx.query();
        let ty = node.ty().ok()?;
        matches!(ty, AnyTsType::TsAnyType(_) | AnyTsType::TsUnknownType(_)).then_some(())
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.syntax().text_trimmed_range(),
                markup! {
                    "Constraining a type parameter to "<Emphasis>"any"</Emphasis>" or "<Emphasis>"unknown"</Emphasis>" is useless."
                },
            )
            .note(markup! {
                "All types are subtypes of "<Emphasis>"any"</Emphasis>" and "<Emphasis>"unknown"</Emphasis>"."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<JsRuleAction> {
        let node = ctx.query();
        let mut mutation = ctx.root().begin();
        let prev = node.syntax().prev_sibling()?;
        mutation.replace_element_discard_trivia(
            prev.clone().into(),
            prev.trim_trailing_trivia()?.into(),
        );
        mutation.remove_node(node.clone());
        Some(JsRuleAction {
            category: ActionCategory::QuickFix,
            applicability: Applicability::Always,
            message: markup! { "Remove the constraint." }.to_owned(),
            mutation,
        })
    }
}
