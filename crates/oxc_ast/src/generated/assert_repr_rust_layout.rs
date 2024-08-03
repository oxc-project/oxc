use crate::ast::*;
macro_rules! wrap {
    ($($tt:tt)*) => {
        // NOTE: disabled!
        // $($tt)*
    };
}
const _: () = {
    {
        assert!(size_of::<BooleanLiteral>() == 12usize);
        assert!(align_of::<BooleanLiteral>() == 4usize);
        wrap!(
            assert!(std::mem::offset_of!(BooleanLiteral, span) == 0usize);
            assert!(std::mem::offset_of!(BooleanLiteral, value) == 8usize);
        );
    }
    {
        assert!(size_of::<NullLiteral>() == 8usize);
        assert!(align_of::<NullLiteral>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(NullLiteral, span) == 0usize););
    }
    {
        assert!(size_of::<NumericLiteral<'static>>() == 40usize);
        assert!(align_of::<NumericLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(NumericLiteral < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(NumericLiteral < 'static >, value) == 24usize);
            assert!(std::mem::offset_of!(NumericLiteral < 'static >, raw) == 0usize);
            assert!(std::mem::offset_of!(NumericLiteral < 'static >, base) == 32usize);
        );
    }
    {
        assert!(size_of::<BigIntLiteral<'static>>() == 32usize);
        assert!(align_of::<BigIntLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BigIntLiteral < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(BigIntLiteral < 'static >, raw) == 0usize);
            assert!(std::mem::offset_of!(BigIntLiteral < 'static >, base) == 24usize);
        );
    }
    {
        assert!(size_of::<RegExpLiteral<'static>>() == 32usize);
        assert!(align_of::<RegExpLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(RegExpLiteral < 'static >, span) == 24usize);
            assert!(std::mem::offset_of!(RegExpLiteral < 'static >, value) == 32usize);
            assert!(std::mem::offset_of!(RegExpLiteral < 'static >, regex) == 0usize);
        );
    }
    {
        assert!(size_of::<RegExp<'static>>() == 24usize);
        assert!(align_of::<RegExp<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(RegExp < 'static >, pattern) == 0usize);
            assert!(std::mem::offset_of!(RegExp < 'static >, flags) == 16usize);
        );
    }
    {
        assert!(size_of::<EmptyObject>() == 0usize);
        assert!(align_of::<EmptyObject>() == 1usize);
        wrap!();
    }
    {
        assert!(size_of::<StringLiteral<'static>>() == 24usize);
        assert!(align_of::<StringLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(StringLiteral < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(StringLiteral < 'static >, value) == 0usize);
        );
    }
    {
        assert!(size_of::<Program<'static>>() == 104usize);
        assert!(align_of::<Program<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Program < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(Program < 'static >, source_type) == 100usize);
            assert!(std::mem::offset_of!(Program < 'static >, hashbang) == 72usize);
            assert!(std::mem::offset_of!(Program < 'static >, directives) == 0usize);
            assert!(std::mem::offset_of!(Program < 'static >, body) == 32usize);
            assert!(std::mem::offset_of!(Program < 'static >, scope_id) == 96usize);
        );
    }
    {
        assert!(size_of::<IdentifierName<'static>>() == 24usize);
        assert!(align_of::<IdentifierName<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(IdentifierName < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(IdentifierName < 'static >, name) == 0usize);
        );
    }
    {
        assert!(size_of::<IdentifierReference<'static>>() == 32usize);
        assert!(align_of::<IdentifierReference<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(IdentifierReference < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(IdentifierReference < 'static >, name)
            == 0usize); assert!(std::mem::offset_of!(IdentifierReference < 'static >,
            reference_id) == 24usize); assert!(std::mem::offset_of!(IdentifierReference <
            'static >, reference_flag) == 28usize);
        );
    }
    {
        assert!(size_of::<BindingIdentifier<'static>>() == 32usize);
        assert!(align_of::<BindingIdentifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BindingIdentifier < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(BindingIdentifier < 'static >, name)
            == 0usize); assert!(std::mem::offset_of!(BindingIdentifier < 'static >,
            symbol_id) == 24usize);
        );
    }
    {
        assert!(size_of::<LabelIdentifier<'static>>() == 24usize);
        assert!(align_of::<LabelIdentifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(LabelIdentifier < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(LabelIdentifier < 'static >, name) == 0usize);
        );
    }
    {
        assert!(size_of::<ThisExpression>() == 8usize);
        assert!(align_of::<ThisExpression>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(ThisExpression, span) == 0usize););
    }
    {
        assert!(size_of::<ArrayExpression<'static>>() == 56usize);
        assert!(align_of::<ArrayExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ArrayExpression < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(ArrayExpression < 'static >, elements) ==
            0usize); assert!(std::mem::offset_of!(ArrayExpression < 'static >,
            trailing_comma) == 40usize);
        );
    }
    {
        assert!(size_of::<Elision>() == 8usize);
        assert!(align_of::<Elision>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(Elision, span) == 0usize););
    }
    {
        assert!(size_of::<ObjectExpression<'static>>() == 56usize);
        assert!(align_of::<ObjectExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ObjectExpression < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(ObjectExpression < 'static >, properties) ==
            0usize); assert!(std::mem::offset_of!(ObjectExpression < 'static >,
            trailing_comma) == 40usize);
        );
    }
    {
        assert!(size_of::<ObjectProperty<'static>>() == 64usize);
        assert!(align_of::<ObjectProperty<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, kind) == 56usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, key) == 0usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, value) == 32usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, init) == 16usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, method) == 57usize);
            assert!(std::mem::offset_of!(ObjectProperty < 'static >, shorthand) ==
            58usize); assert!(std::mem::offset_of!(ObjectProperty < 'static >, computed)
            == 59usize);
        );
    }
    {
        assert!(size_of::<TemplateLiteral<'static>>() == 72usize);
        assert!(align_of::<TemplateLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TemplateLiteral < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(TemplateLiteral < 'static >, quasis) == 0usize);
            assert!(std::mem::offset_of!(TemplateLiteral < 'static >, expressions) ==
            32usize);
        );
    }
    {
        assert!(size_of::<TaggedTemplateExpression<'static>>() == 104usize);
        assert!(align_of::<TaggedTemplateExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TaggedTemplateExpression < 'static >, span) ==
            88usize); assert!(std::mem::offset_of!(TaggedTemplateExpression < 'static >,
            tag) == 0usize); assert!(std::mem::offset_of!(TaggedTemplateExpression <
            'static >, quasi) == 16usize);
            assert!(std::mem::offset_of!(TaggedTemplateExpression < 'static >,
            type_parameters) == 96usize);
        );
    }
    {
        assert!(size_of::<TemplateElement<'static>>() == 48usize);
        assert!(align_of::<TemplateElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TemplateElement < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TemplateElement < 'static >, tail) == 40usize);
            assert!(std::mem::offset_of!(TemplateElement < 'static >, value) == 0usize);
        );
    }
    {
        assert!(size_of::<TemplateElementValue<'static>>() == 32usize);
        assert!(align_of::<TemplateElementValue<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TemplateElementValue < 'static >, raw) ==
            0usize); assert!(std::mem::offset_of!(TemplateElementValue < 'static >,
            cooked) == 16usize);
        );
    }
    {
        assert!(size_of::<ComputedMemberExpression<'static>>() == 48usize);
        assert!(align_of::<ComputedMemberExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ComputedMemberExpression < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(ComputedMemberExpression < 'static >,
            object) == 0usize); assert!(std::mem::offset_of!(ComputedMemberExpression <
            'static >, expression) == 16usize);
            assert!(std::mem::offset_of!(ComputedMemberExpression < 'static >, optional)
            == 40usize);
        );
    }
    {
        assert!(size_of::<StaticMemberExpression<'static>>() == 56usize);
        assert!(align_of::<StaticMemberExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(StaticMemberExpression < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(StaticMemberExpression < 'static >,
            object) == 0usize); assert!(std::mem::offset_of!(StaticMemberExpression <
            'static >, property) == 16usize);
            assert!(std::mem::offset_of!(StaticMemberExpression < 'static >, optional) ==
            48usize);
        );
    }
    {
        assert!(size_of::<PrivateFieldExpression<'static>>() == 56usize);
        assert!(align_of::<PrivateFieldExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(PrivateFieldExpression < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(PrivateFieldExpression < 'static >,
            object) == 0usize); assert!(std::mem::offset_of!(PrivateFieldExpression <
            'static >, field) == 16usize);
            assert!(std::mem::offset_of!(PrivateFieldExpression < 'static >, optional) ==
            48usize);
        );
    }
    {
        assert!(size_of::<CallExpression<'static>>() == 72usize);
        assert!(align_of::<CallExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(CallExpression < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(CallExpression < 'static >, arguments) ==
            16usize); assert!(std::mem::offset_of!(CallExpression < 'static >, callee) ==
            0usize); assert!(std::mem::offset_of!(CallExpression < 'static >,
            type_parameters) == 56usize); assert!(std::mem::offset_of!(CallExpression <
            'static >, optional) == 64usize);
        );
    }
    {
        assert!(size_of::<NewExpression<'static>>() == 64usize);
        assert!(align_of::<NewExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(NewExpression < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(NewExpression < 'static >, callee) == 0usize);
            assert!(std::mem::offset_of!(NewExpression < 'static >, arguments) ==
            16usize); assert!(std::mem::offset_of!(NewExpression < 'static >,
            type_parameters) == 56usize);
        );
    }
    {
        assert!(size_of::<MetaProperty<'static>>() == 56usize);
        assert!(align_of::<MetaProperty<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(MetaProperty < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(MetaProperty < 'static >, meta) == 0usize);
            assert!(std::mem::offset_of!(MetaProperty < 'static >, property) == 24usize);
        );
    }
    {
        assert!(size_of::<SpreadElement<'static>>() == 24usize);
        assert!(align_of::<SpreadElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(SpreadElement < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(SpreadElement < 'static >, argument) == 0usize);
        );
    }
    {
        assert!(size_of::<UpdateExpression<'static>>() == 32usize);
        assert!(align_of::<UpdateExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(UpdateExpression < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(UpdateExpression < 'static >, operator) ==
            24usize); assert!(std::mem::offset_of!(UpdateExpression < 'static >, prefix)
            == 25usize); assert!(std::mem::offset_of!(UpdateExpression < 'static >,
            argument) == 0usize);
        );
    }
    {
        assert!(size_of::<UnaryExpression<'static>>() == 32usize);
        assert!(align_of::<UnaryExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(UnaryExpression < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(UnaryExpression < 'static >, operator) ==
            24usize); assert!(std::mem::offset_of!(UnaryExpression < 'static >, argument)
            == 0usize);
        );
    }
    {
        assert!(size_of::<BinaryExpression<'static>>() == 48usize);
        assert!(align_of::<BinaryExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BinaryExpression < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(BinaryExpression < 'static >, left) == 0usize);
            assert!(std::mem::offset_of!(BinaryExpression < 'static >, operator) ==
            40usize); assert!(std::mem::offset_of!(BinaryExpression < 'static >, right)
            == 16usize);
        );
    }
    {
        assert!(size_of::<PrivateInExpression<'static>>() == 56usize);
        assert!(align_of::<PrivateInExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(PrivateInExpression < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(PrivateInExpression < 'static >, left)
            == 16usize); assert!(std::mem::offset_of!(PrivateInExpression < 'static >,
            operator) == 48usize); assert!(std::mem::offset_of!(PrivateInExpression <
            'static >, right) == 0usize);
        );
    }
    {
        assert!(size_of::<LogicalExpression<'static>>() == 48usize);
        assert!(align_of::<LogicalExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(LogicalExpression < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(LogicalExpression < 'static >, left)
            == 0usize); assert!(std::mem::offset_of!(LogicalExpression < 'static >,
            operator) == 40usize); assert!(std::mem::offset_of!(LogicalExpression <
            'static >, right) == 16usize);
        );
    }
    {
        assert!(size_of::<ConditionalExpression<'static>>() == 56usize);
        assert!(align_of::<ConditionalExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ConditionalExpression < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(ConditionalExpression < 'static >,
            test) == 0usize); assert!(std::mem::offset_of!(ConditionalExpression <
            'static >, consequent) == 16usize);
            assert!(std::mem::offset_of!(ConditionalExpression < 'static >, alternate) ==
            32usize);
        );
    }
    {
        assert!(size_of::<AssignmentExpression<'static>>() == 48usize);
        assert!(align_of::<AssignmentExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentExpression < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(AssignmentExpression < 'static >,
            operator) == 40usize); assert!(std::mem::offset_of!(AssignmentExpression <
            'static >, left) == 0usize);
            assert!(std::mem::offset_of!(AssignmentExpression < 'static >, right) ==
            16usize);
        );
    }
    {
        assert!(size_of::<ArrayAssignmentTarget<'static>>() == 80usize);
        assert!(align_of::<ArrayAssignmentTarget<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ArrayAssignmentTarget < 'static >, span) ==
            56usize); assert!(std::mem::offset_of!(ArrayAssignmentTarget < 'static >,
            elements) == 24usize); assert!(std::mem::offset_of!(ArrayAssignmentTarget <
            'static >, rest) == 0usize);
            assert!(std::mem::offset_of!(ArrayAssignmentTarget < 'static >,
            trailing_comma) == 64usize);
        );
    }
    {
        assert!(size_of::<ObjectAssignmentTarget<'static>>() == 64usize);
        assert!(align_of::<ObjectAssignmentTarget<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ObjectAssignmentTarget < 'static >, span) ==
            56usize); assert!(std::mem::offset_of!(ObjectAssignmentTarget < 'static >,
            properties) == 24usize); assert!(std::mem::offset_of!(ObjectAssignmentTarget
            < 'static >, rest) == 0usize);
        );
    }
    {
        assert!(size_of::<AssignmentTargetRest<'static>>() == 24usize);
        assert!(align_of::<AssignmentTargetRest<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentTargetRest < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(AssignmentTargetRest < 'static >,
            target) == 0usize);
        );
    }
    {
        assert!(size_of::<AssignmentTargetWithDefault<'static>>() == 40usize);
        assert!(align_of::<AssignmentTargetWithDefault<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentTargetWithDefault < 'static >, span)
            == 32usize); assert!(std::mem::offset_of!(AssignmentTargetWithDefault <
            'static >, binding) == 0usize);
            assert!(std::mem::offset_of!(AssignmentTargetWithDefault < 'static >, init)
            == 16usize);
        );
    }
    {
        assert!(size_of::<AssignmentTargetPropertyIdentifier<'static>>() == 56usize);
        assert!(align_of::<AssignmentTargetPropertyIdentifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentTargetPropertyIdentifier < 'static >,
            span) == 48usize);
            assert!(std::mem::offset_of!(AssignmentTargetPropertyIdentifier < 'static >,
            binding) == 16usize);
            assert!(std::mem::offset_of!(AssignmentTargetPropertyIdentifier < 'static >,
            init) == 0usize);
        );
    }
    {
        assert!(size_of::<AssignmentTargetPropertyProperty<'static>>() == 40usize);
        assert!(align_of::<AssignmentTargetPropertyProperty<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentTargetPropertyProperty < 'static >,
            span) == 32usize);
            assert!(std::mem::offset_of!(AssignmentTargetPropertyProperty < 'static >,
            name) == 16usize);
            assert!(std::mem::offset_of!(AssignmentTargetPropertyProperty < 'static >,
            binding) == 0usize);
        );
    }
    {
        assert!(size_of::<SequenceExpression<'static>>() == 40usize);
        assert!(align_of::<SequenceExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(SequenceExpression < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(SequenceExpression < 'static >,
            expressions) == 0usize);
        );
    }
    {
        assert!(size_of::<Super>() == 8usize);
        assert!(align_of::<Super>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(Super, span) == 0usize););
    }
    {
        assert!(size_of::<AwaitExpression<'static>>() == 24usize);
        assert!(align_of::<AwaitExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AwaitExpression < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(AwaitExpression < 'static >, argument) ==
            0usize);
        );
    }
    {
        assert!(size_of::<ChainExpression<'static>>() == 24usize);
        assert!(align_of::<ChainExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ChainExpression < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(ChainExpression < 'static >, expression) ==
            0usize);
        );
    }
    {
        assert!(size_of::<ParenthesizedExpression<'static>>() == 24usize);
        assert!(align_of::<ParenthesizedExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ParenthesizedExpression < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(ParenthesizedExpression < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<Directive<'static>>() == 48usize);
        assert!(align_of::<Directive<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Directive < 'static >, span) == 40usize);
            assert!(std::mem::offset_of!(Directive < 'static >, expression) == 0usize);
            assert!(std::mem::offset_of!(Directive < 'static >, directive) == 24usize);
        );
    }
    {
        assert!(size_of::<Hashbang<'static>>() == 24usize);
        assert!(align_of::<Hashbang<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Hashbang < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(Hashbang < 'static >, value) == 0usize);
        );
    }
    {
        assert!(size_of::<BlockStatement<'static>>() == 48usize);
        assert!(align_of::<BlockStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BlockStatement < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(BlockStatement < 'static >, body) == 0usize);
            assert!(std::mem::offset_of!(BlockStatement < 'static >, scope_id) ==
            40usize);
        );
    }
    {
        assert!(size_of::<VariableDeclaration<'static>>() == 48usize);
        assert!(align_of::<VariableDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(VariableDeclaration < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(VariableDeclaration < 'static >, kind)
            == 40usize); assert!(std::mem::offset_of!(VariableDeclaration < 'static >,
            declarations) == 0usize); assert!(std::mem::offset_of!(VariableDeclaration <
            'static >, declare) == 41usize);
        );
    }
    {
        assert!(size_of::<VariableDeclarator<'static>>() == 64usize);
        assert!(align_of::<VariableDeclarator<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(VariableDeclarator < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(VariableDeclarator < 'static >, kind)
            == 56usize); assert!(std::mem::offset_of!(VariableDeclarator < 'static >, id)
            == 24usize); assert!(std::mem::offset_of!(VariableDeclarator < 'static >,
            init) == 0usize); assert!(std::mem::offset_of!(VariableDeclarator < 'static
            >, definite) == 57usize);
        );
    }
    {
        assert!(size_of::<UsingDeclaration<'static>>() == 48usize);
        assert!(align_of::<UsingDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(UsingDeclaration < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(UsingDeclaration < 'static >, is_await) ==
            40usize); assert!(std::mem::offset_of!(UsingDeclaration < 'static >,
            declarations) == 0usize);
        );
    }
    {
        assert!(size_of::<EmptyStatement>() == 8usize);
        assert!(align_of::<EmptyStatement>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(EmptyStatement, span) == 0usize););
    }
    {
        assert!(size_of::<ExpressionStatement<'static>>() == 24usize);
        assert!(align_of::<ExpressionStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ExpressionStatement < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(ExpressionStatement < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<IfStatement<'static>>() == 56usize);
        assert!(align_of::<IfStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(IfStatement < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(IfStatement < 'static >, test) == 0usize);
            assert!(std::mem::offset_of!(IfStatement < 'static >, consequent) ==
            16usize); assert!(std::mem::offset_of!(IfStatement < 'static >, alternate) ==
            32usize);
        );
    }
    {
        assert!(size_of::<DoWhileStatement<'static>>() == 40usize);
        assert!(align_of::<DoWhileStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(DoWhileStatement < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(DoWhileStatement < 'static >, body) == 16usize);
            assert!(std::mem::offset_of!(DoWhileStatement < 'static >, test) == 0usize);
        );
    }
    {
        assert!(size_of::<WhileStatement<'static>>() == 40usize);
        assert!(align_of::<WhileStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(WhileStatement < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(WhileStatement < 'static >, test) == 0usize);
            assert!(std::mem::offset_of!(WhileStatement < 'static >, body) == 16usize);
        );
    }
    {
        assert!(size_of::<ForStatement<'static>>() == 80usize);
        assert!(align_of::<ForStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ForStatement < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(ForStatement < 'static >, init) == 32usize);
            assert!(std::mem::offset_of!(ForStatement < 'static >, test) == 0usize);
            assert!(std::mem::offset_of!(ForStatement < 'static >, update) == 16usize);
            assert!(std::mem::offset_of!(ForStatement < 'static >, body) == 48usize);
            assert!(std::mem::offset_of!(ForStatement < 'static >, scope_id) == 72usize);
        );
    }
    {
        assert!(size_of::<ForInStatement<'static>>() == 64usize);
        assert!(align_of::<ForInStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ForInStatement < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(ForInStatement < 'static >, left) == 0usize);
            assert!(std::mem::offset_of!(ForInStatement < 'static >, right) == 16usize);
            assert!(std::mem::offset_of!(ForInStatement < 'static >, body) == 32usize);
            assert!(std::mem::offset_of!(ForInStatement < 'static >, scope_id) ==
            56usize);
        );
    }
    {
        assert!(size_of::<ForOfStatement<'static>>() == 64usize);
        assert!(align_of::<ForOfStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ForOfStatement < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(ForOfStatement < 'static >, r#await) ==
            60usize); assert!(std::mem::offset_of!(ForOfStatement < 'static >, left) ==
            0usize); assert!(std::mem::offset_of!(ForOfStatement < 'static >, right) ==
            16usize); assert!(std::mem::offset_of!(ForOfStatement < 'static >, body) ==
            32usize); assert!(std::mem::offset_of!(ForOfStatement < 'static >, scope_id)
            == 56usize);
        );
    }
    {
        assert!(size_of::<ContinueStatement<'static>>() == 32usize);
        assert!(align_of::<ContinueStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ContinueStatement < 'static >, span) == 0usize);
            assert!(std::mem::offset_of!(ContinueStatement < 'static >, label) ==
            8usize);
        );
    }
    {
        assert!(size_of::<BreakStatement<'static>>() == 32usize);
        assert!(align_of::<BreakStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BreakStatement < 'static >, span) == 0usize);
            assert!(std::mem::offset_of!(BreakStatement < 'static >, label) == 8usize);
        );
    }
    {
        assert!(size_of::<ReturnStatement<'static>>() == 24usize);
        assert!(align_of::<ReturnStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ReturnStatement < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(ReturnStatement < 'static >, argument) ==
            0usize);
        );
    }
    {
        assert!(size_of::<WithStatement<'static>>() == 40usize);
        assert!(align_of::<WithStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(WithStatement < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(WithStatement < 'static >, object) == 0usize);
            assert!(std::mem::offset_of!(WithStatement < 'static >, body) == 16usize);
        );
    }
    {
        assert!(size_of::<SwitchStatement<'static>>() == 64usize);
        assert!(align_of::<SwitchStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(SwitchStatement < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(SwitchStatement < 'static >, discriminant) ==
            0usize); assert!(std::mem::offset_of!(SwitchStatement < 'static >, cases) ==
            16usize); assert!(std::mem::offset_of!(SwitchStatement < 'static >, scope_id)
            == 56usize);
        );
    }
    {
        assert!(size_of::<SwitchCase<'static>>() == 56usize);
        assert!(align_of::<SwitchCase<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(SwitchCase < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(SwitchCase < 'static >, test) == 0usize);
            assert!(std::mem::offset_of!(SwitchCase < 'static >, consequent) == 16usize);
        );
    }
    {
        assert!(size_of::<LabeledStatement<'static>>() == 48usize);
        assert!(align_of::<LabeledStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(LabeledStatement < 'static >, span) == 40usize);
            assert!(std::mem::offset_of!(LabeledStatement < 'static >, label) ==
            16usize); assert!(std::mem::offset_of!(LabeledStatement < 'static >, body) ==
            0usize);
        );
    }
    {
        assert!(size_of::<ThrowStatement<'static>>() == 24usize);
        assert!(align_of::<ThrowStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ThrowStatement < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(ThrowStatement < 'static >, argument) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TryStatement<'static>>() == 32usize);
        assert!(align_of::<TryStatement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TryStatement < 'static >, span) == 8usize);
            assert!(std::mem::offset_of!(TryStatement < 'static >, block) == 0usize);
            assert!(std::mem::offset_of!(TryStatement < 'static >, handler) == 16usize);
            assert!(std::mem::offset_of!(TryStatement < 'static >, finalizer) ==
            24usize);
        );
    }
    {
        assert!(size_of::<CatchClause<'static>>() == 64usize);
        assert!(align_of::<CatchClause<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(CatchClause < 'static >, span) == 0usize);
            assert!(std::mem::offset_of!(CatchClause < 'static >, param) == 16usize);
            assert!(std::mem::offset_of!(CatchClause < 'static >, body) == 8usize);
            assert!(std::mem::offset_of!(CatchClause < 'static >, scope_id) == 56usize);
        );
    }
    {
        assert!(size_of::<CatchParameter<'static>>() == 40usize);
        assert!(align_of::<CatchParameter<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(CatchParameter < 'static >, span) == 0usize);
            assert!(std::mem::offset_of!(CatchParameter < 'static >, pattern) == 8usize);
        );
    }
    {
        assert!(size_of::<DebuggerStatement>() == 8usize);
        assert!(align_of::<DebuggerStatement>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(DebuggerStatement, span) == 0usize););
    }
    {
        assert!(size_of::<BindingPattern<'static>>() == 32usize);
        assert!(align_of::<BindingPattern<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BindingPattern < 'static >, kind) == 0usize);
            assert!(std::mem::offset_of!(BindingPattern < 'static >, type_annotation) ==
            16usize); assert!(std::mem::offset_of!(BindingPattern < 'static >, optional)
            == 24usize);
        );
    }
    {
        assert!(size_of::<AssignmentPattern<'static>>() == 56usize);
        assert!(align_of::<AssignmentPattern<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AssignmentPattern < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(AssignmentPattern < 'static >, left)
            == 24usize); assert!(std::mem::offset_of!(AssignmentPattern < 'static >,
            right) == 0usize);
        );
    }
    {
        assert!(size_of::<ObjectPattern<'static>>() == 48usize);
        assert!(align_of::<ObjectPattern<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ObjectPattern < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(ObjectPattern < 'static >, properties) ==
            0usize); assert!(std::mem::offset_of!(ObjectPattern < 'static >, rest) ==
            40usize);
        );
    }
    {
        assert!(size_of::<BindingProperty<'static>>() == 64usize);
        assert!(align_of::<BindingProperty<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BindingProperty < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(BindingProperty < 'static >, key) == 0usize);
            assert!(std::mem::offset_of!(BindingProperty < 'static >, value) == 24usize);
            assert!(std::mem::offset_of!(BindingProperty < 'static >, shorthand) ==
            56usize); assert!(std::mem::offset_of!(BindingProperty < 'static >, computed)
            == 57usize);
        );
    }
    {
        assert!(size_of::<ArrayPattern<'static>>() == 48usize);
        assert!(align_of::<ArrayPattern<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ArrayPattern < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(ArrayPattern < 'static >, elements) == 0usize);
            assert!(std::mem::offset_of!(ArrayPattern < 'static >, rest) == 40usize);
        );
    }
    {
        assert!(size_of::<BindingRestElement<'static>>() == 40usize);
        assert!(align_of::<BindingRestElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(BindingRestElement < 'static >, span) ==
            0usize); assert!(std::mem::offset_of!(BindingRestElement < 'static >,
            argument) == 8usize);
        );
    }
    {
        assert!(size_of::<Function<'static>>() == 120usize);
        assert!(align_of::<Function<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Function < 'static >, r#type) == 116usize);
            assert!(std::mem::offset_of!(Function < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(Function < 'static >, id) == 0usize);
            assert!(std::mem::offset_of!(Function < 'static >, generator) == 117usize);
            assert!(std::mem::offset_of!(Function < 'static >, r#async) == 118usize);
            assert!(std::mem::offset_of!(Function < 'static >, declare) == 119usize);
            assert!(std::mem::offset_of!(Function < 'static >, type_parameters) ==
            40usize); assert!(std::mem::offset_of!(Function < 'static >, this_param) ==
            48usize); assert!(std::mem::offset_of!(Function < 'static >, params) ==
            104usize); assert!(std::mem::offset_of!(Function < 'static >, return_type) ==
            88usize); assert!(std::mem::offset_of!(Function < 'static >, body) ==
            96usize); assert!(std::mem::offset_of!(Function < 'static >, scope_id) ==
            112usize);
        );
    }
    {
        assert!(size_of::<FormalParameters<'static>>() == 56usize);
        assert!(align_of::<FormalParameters<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(FormalParameters < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(FormalParameters < 'static >, kind) == 48usize);
            assert!(std::mem::offset_of!(FormalParameters < 'static >, items) == 0usize);
            assert!(std::mem::offset_of!(FormalParameters < 'static >, rest) == 40usize);
        );
    }
    {
        assert!(size_of::<FormalParameter<'static>>() == 80usize);
        assert!(align_of::<FormalParameter<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(FormalParameter < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(FormalParameter < 'static >, decorators) ==
            0usize); assert!(std::mem::offset_of!(FormalParameter < 'static >, pattern)
            == 40usize); assert!(std::mem::offset_of!(FormalParameter < 'static >,
            accessibility) == 72usize); assert!(std::mem::offset_of!(FormalParameter <
            'static >, readonly) == 73usize);
            assert!(std::mem::offset_of!(FormalParameter < 'static >, r#override) ==
            74usize);
        );
    }
    {
        assert!(size_of::<FunctionBody<'static>>() == 72usize);
        assert!(align_of::<FunctionBody<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(FunctionBody < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(FunctionBody < 'static >, directives) ==
            0usize); assert!(std::mem::offset_of!(FunctionBody < 'static >, statements)
            == 32usize);
        );
    }
    {
        assert!(size_of::<ArrowFunctionExpression<'static>>() == 48usize);
        assert!(align_of::<ArrowFunctionExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >, span) ==
            0usize); assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >,
            expression) == 44usize); assert!(std::mem::offset_of!(ArrowFunctionExpression
            < 'static >, r#async) == 45usize);
            assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >,
            type_parameters) == 8usize);
            assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >, params) ==
            24usize); assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >,
            return_type) == 16usize);
            assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >, body) ==
            32usize); assert!(std::mem::offset_of!(ArrowFunctionExpression < 'static >,
            scope_id) == 40usize);
        );
    }
    {
        assert!(size_of::<YieldExpression<'static>>() == 32usize);
        assert!(align_of::<YieldExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(YieldExpression < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(YieldExpression < 'static >, delegate) ==
            24usize); assert!(std::mem::offset_of!(YieldExpression < 'static >, argument)
            == 0usize);
        );
    }
    {
        assert!(size_of::<Class<'static>>() == 152usize);
        assert!(align_of::<Class<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Class < 'static >, r#type) == 148usize);
            assert!(std::mem::offset_of!(Class < 'static >, span) == 112usize);
            assert!(std::mem::offset_of!(Class < 'static >, decorators) == 64usize);
            assert!(std::mem::offset_of!(Class < 'static >, id) == 0usize);
            assert!(std::mem::offset_of!(Class < 'static >, type_parameters) ==
            120usize); assert!(std::mem::offset_of!(Class < 'static >, super_class) ==
            96usize); assert!(std::mem::offset_of!(Class < 'static >,
            super_type_parameters) == 128usize); assert!(std::mem::offset_of!(Class <
            'static >, implements) == 32usize); assert!(std::mem::offset_of!(Class <
            'static >, body) == 136usize); assert!(std::mem::offset_of!(Class < 'static
            >, r#abstract) == 149usize); assert!(std::mem::offset_of!(Class < 'static >,
            declare) == 150usize); assert!(std::mem::offset_of!(Class < 'static >,
            scope_id) == 144usize);
        );
    }
    {
        assert!(size_of::<ClassBody<'static>>() == 40usize);
        assert!(align_of::<ClassBody<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ClassBody < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(ClassBody < 'static >, body) == 0usize);
        );
    }
    {
        assert!(size_of::<MethodDefinition<'static>>() == 72usize);
        assert!(align_of::<MethodDefinition<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(MethodDefinition < 'static >, r#type) ==
            66usize); assert!(std::mem::offset_of!(MethodDefinition < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(MethodDefinition < 'static >,
            decorators) == 0usize); assert!(std::mem::offset_of!(MethodDefinition <
            'static >, key) == 32usize); assert!(std::mem::offset_of!(MethodDefinition <
            'static >, value) == 56usize); assert!(std::mem::offset_of!(MethodDefinition
            < 'static >, kind) == 64usize); assert!(std::mem::offset_of!(MethodDefinition
            < 'static >, computed) == 67usize);
            assert!(std::mem::offset_of!(MethodDefinition < 'static >, r#static) ==
            68usize); assert!(std::mem::offset_of!(MethodDefinition < 'static >,
            r#override) == 69usize); assert!(std::mem::offset_of!(MethodDefinition <
            'static >, optional) == 70usize);
            assert!(std::mem::offset_of!(MethodDefinition < 'static >, accessibility) ==
            65usize);
        );
    }
    {
        assert!(size_of::<PropertyDefinition<'static>>() == 96usize);
        assert!(align_of::<PropertyDefinition<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(PropertyDefinition < 'static >, r#type) ==
            81usize); assert!(std::mem::offset_of!(PropertyDefinition < 'static >, span)
            == 64usize); assert!(std::mem::offset_of!(PropertyDefinition < 'static >,
            decorators) == 0usize); assert!(std::mem::offset_of!(PropertyDefinition <
            'static >, key) == 32usize); assert!(std::mem::offset_of!(PropertyDefinition
            < 'static >, value) == 48usize);
            assert!(std::mem::offset_of!(PropertyDefinition < 'static >, computed) ==
            82usize); assert!(std::mem::offset_of!(PropertyDefinition < 'static >,
            r#static) == 83usize); assert!(std::mem::offset_of!(PropertyDefinition <
            'static >, declare) == 84usize);
            assert!(std::mem::offset_of!(PropertyDefinition < 'static >, r#override) ==
            85usize); assert!(std::mem::offset_of!(PropertyDefinition < 'static >,
            optional) == 86usize); assert!(std::mem::offset_of!(PropertyDefinition <
            'static >, definite) == 87usize);
            assert!(std::mem::offset_of!(PropertyDefinition < 'static >, readonly) ==
            88usize); assert!(std::mem::offset_of!(PropertyDefinition < 'static >,
            type_annotation) == 72usize); assert!(std::mem::offset_of!(PropertyDefinition
            < 'static >, accessibility) == 80usize);
        );
    }
    {
        assert!(size_of::<PrivateIdentifier<'static>>() == 24usize);
        assert!(align_of::<PrivateIdentifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(PrivateIdentifier < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(PrivateIdentifier < 'static >, name)
            == 0usize);
        );
    }
    {
        assert!(size_of::<StaticBlock<'static>>() == 48usize);
        assert!(align_of::<StaticBlock<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(StaticBlock < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(StaticBlock < 'static >, body) == 0usize);
            assert!(std::mem::offset_of!(StaticBlock < 'static >, scope_id) == 40usize);
        );
    }
    {
        assert!(size_of::<AccessorProperty<'static>>() == 80usize);
        assert!(align_of::<AccessorProperty<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(AccessorProperty < 'static >, r#type) ==
            72usize); assert!(std::mem::offset_of!(AccessorProperty < 'static >, span) ==
            64usize); assert!(std::mem::offset_of!(AccessorProperty < 'static >,
            decorators) == 0usize); assert!(std::mem::offset_of!(AccessorProperty <
            'static >, key) == 32usize); assert!(std::mem::offset_of!(AccessorProperty <
            'static >, value) == 48usize); assert!(std::mem::offset_of!(AccessorProperty
            < 'static >, computed) == 73usize);
            assert!(std::mem::offset_of!(AccessorProperty < 'static >, r#static) ==
            74usize);
        );
    }
    {
        assert!(size_of::<ImportExpression<'static>>() == 56usize);
        assert!(align_of::<ImportExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportExpression < 'static >, span) == 48usize);
            assert!(std::mem::offset_of!(ImportExpression < 'static >, source) ==
            0usize); assert!(std::mem::offset_of!(ImportExpression < 'static >,
            arguments) == 16usize);
        );
    }
    {
        assert!(size_of::<ImportDeclaration<'static>>() == 136usize);
        assert!(align_of::<ImportDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportDeclaration < 'static >, span) ==
            24usize); assert!(std::mem::offset_of!(ImportDeclaration < 'static >,
            specifiers) == 32usize); assert!(std::mem::offset_of!(ImportDeclaration <
            'static >, source) == 0usize); assert!(std::mem::offset_of!(ImportDeclaration
            < 'static >, with_clause) == 64usize);
            assert!(std::mem::offset_of!(ImportDeclaration < 'static >, import_kind) ==
            128usize);
        );
    }
    {
        assert!(size_of::<ImportSpecifier<'static>>() == 88usize);
        assert!(align_of::<ImportSpecifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportSpecifier < 'static >, span) == 72usize);
            assert!(std::mem::offset_of!(ImportSpecifier < 'static >, imported) ==
            0usize); assert!(std::mem::offset_of!(ImportSpecifier < 'static >, local) ==
            40usize); assert!(std::mem::offset_of!(ImportSpecifier < 'static >,
            import_kind) == 80usize);
        );
    }
    {
        assert!(size_of::<ImportDefaultSpecifier<'static>>() == 40usize);
        assert!(align_of::<ImportDefaultSpecifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportDefaultSpecifier < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(ImportDefaultSpecifier < 'static >,
            local) == 0usize);
        );
    }
    {
        assert!(size_of::<ImportNamespaceSpecifier<'static>>() == 40usize);
        assert!(align_of::<ImportNamespaceSpecifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportNamespaceSpecifier < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(ImportNamespaceSpecifier < 'static >,
            local) == 0usize);
        );
    }
    {
        assert!(size_of::<WithClause<'static>>() == 64usize);
        assert!(align_of::<WithClause<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(WithClause < 'static >, span) == 56usize);
            assert!(std::mem::offset_of!(WithClause < 'static >, attributes_keyword) ==
            0usize); assert!(std::mem::offset_of!(WithClause < 'static >, with_entries)
            == 24usize);
        );
    }
    {
        assert!(size_of::<ImportAttribute<'static>>() == 64usize);
        assert!(align_of::<ImportAttribute<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ImportAttribute < 'static >, span) == 56usize);
            assert!(std::mem::offset_of!(ImportAttribute < 'static >, key) == 0usize);
            assert!(std::mem::offset_of!(ImportAttribute < 'static >, value) == 32usize);
        );
    }
    {
        assert!(size_of::<ExportNamedDeclaration<'static>>() == 152usize);
        assert!(align_of::<ExportNamedDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ExportNamedDeclaration < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(ExportNamedDeclaration < 'static >,
            declaration) == 0usize); assert!(std::mem::offset_of!(ExportNamedDeclaration
            < 'static >, specifiers) == 16usize);
            assert!(std::mem::offset_of!(ExportNamedDeclaration < 'static >, source) ==
            56usize); assert!(std::mem::offset_of!(ExportNamedDeclaration < 'static >,
            export_kind) == 144usize);
            assert!(std::mem::offset_of!(ExportNamedDeclaration < 'static >, with_clause)
            == 80usize);
        );
    }
    {
        assert!(size_of::<ExportDefaultDeclaration<'static>>() == 64usize);
        assert!(align_of::<ExportDefaultDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ExportDefaultDeclaration < 'static >, span) ==
            56usize); assert!(std::mem::offset_of!(ExportDefaultDeclaration < 'static >,
            declaration) == 40usize);
            assert!(std::mem::offset_of!(ExportDefaultDeclaration < 'static >, exported)
            == 0usize);
        );
    }
    {
        assert!(size_of::<ExportAllDeclaration<'static>>() == 144usize);
        assert!(align_of::<ExportAllDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ExportAllDeclaration < 'static >, span) ==
            64usize); assert!(std::mem::offset_of!(ExportAllDeclaration < 'static >,
            exported) == 0usize); assert!(std::mem::offset_of!(ExportAllDeclaration <
            'static >, source) == 40usize);
            assert!(std::mem::offset_of!(ExportAllDeclaration < 'static >, with_clause)
            == 72usize); assert!(std::mem::offset_of!(ExportAllDeclaration < 'static >,
            export_kind) == 136usize);
        );
    }
    {
        assert!(size_of::<ExportSpecifier<'static>>() == 96usize);
        assert!(align_of::<ExportSpecifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(ExportSpecifier < 'static >, span) == 80usize);
            assert!(std::mem::offset_of!(ExportSpecifier < 'static >, local) == 0usize);
            assert!(std::mem::offset_of!(ExportSpecifier < 'static >, exported) ==
            40usize); assert!(std::mem::offset_of!(ExportSpecifier < 'static >,
            export_kind) == 88usize);
        );
    }
    {
        assert!(size_of::<TSThisParameter<'static>>() == 40usize);
        assert!(align_of::<TSThisParameter<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSThisParameter < 'static >, span) == 24usize);
            assert!(std::mem::offset_of!(TSThisParameter < 'static >, this) == 0usize);
            assert!(std::mem::offset_of!(TSThisParameter < 'static >, type_annotation) ==
            32usize);
        );
    }
    {
        assert!(size_of::<TSEnumDeclaration<'static>>() == 80usize);
        assert!(align_of::<TSEnumDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSEnumDeclaration < 'static >, span) ==
            64usize); assert!(std::mem::offset_of!(TSEnumDeclaration < 'static >, id) ==
            0usize); assert!(std::mem::offset_of!(TSEnumDeclaration < 'static >, members)
            == 32usize); assert!(std::mem::offset_of!(TSEnumDeclaration < 'static >,
            r#const) == 76usize); assert!(std::mem::offset_of!(TSEnumDeclaration <
            'static >, declare) == 77usize);
            assert!(std::mem::offset_of!(TSEnumDeclaration < 'static >, scope_id) ==
            72usize);
        );
    }
    {
        assert!(size_of::<TSEnumMember<'static>>() == 40usize);
        assert!(align_of::<TSEnumMember<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSEnumMember < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSEnumMember < 'static >, id) == 16usize);
            assert!(std::mem::offset_of!(TSEnumMember < 'static >, initializer) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSTypeAnnotation<'static>>() == 24usize);
        assert!(align_of::<TSTypeAnnotation<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeAnnotation < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSTypeAnnotation < 'static >, type_annotation)
            == 0usize);
        );
    }
    {
        assert!(size_of::<TSLiteralType<'static>>() == 24usize);
        assert!(align_of::<TSLiteralType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSLiteralType < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSLiteralType < 'static >, literal) == 0usize);
        );
    }
    {
        assert!(size_of::<TSConditionalType<'static>>() == 80usize);
        assert!(align_of::<TSConditionalType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSConditionalType < 'static >, span) ==
            64usize); assert!(std::mem::offset_of!(TSConditionalType < 'static >,
            check_type) == 0usize); assert!(std::mem::offset_of!(TSConditionalType <
            'static >, extends_type) == 16usize);
            assert!(std::mem::offset_of!(TSConditionalType < 'static >, true_type) ==
            32usize); assert!(std::mem::offset_of!(TSConditionalType < 'static >,
            false_type) == 48usize); assert!(std::mem::offset_of!(TSConditionalType <
            'static >, scope_id) == 72usize);
        );
    }
    {
        assert!(size_of::<TSUnionType<'static>>() == 40usize);
        assert!(align_of::<TSUnionType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSUnionType < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSUnionType < 'static >, types) == 0usize);
        );
    }
    {
        assert!(size_of::<TSIntersectionType<'static>>() == 40usize);
        assert!(align_of::<TSIntersectionType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSIntersectionType < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(TSIntersectionType < 'static >, types)
            == 0usize);
        );
    }
    {
        assert!(size_of::<TSParenthesizedType<'static>>() == 24usize);
        assert!(align_of::<TSParenthesizedType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSParenthesizedType < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSParenthesizedType < 'static >,
            type_annotation) == 0usize);
        );
    }
    {
        assert!(size_of::<TSTypeOperator<'static>>() == 32usize);
        assert!(align_of::<TSTypeOperator<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeOperator < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSTypeOperator < 'static >, operator) ==
            24usize); assert!(std::mem::offset_of!(TSTypeOperator < 'static >,
            type_annotation) == 0usize);
        );
    }
    {
        assert!(size_of::<TSArrayType<'static>>() == 24usize);
        assert!(align_of::<TSArrayType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSArrayType < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSArrayType < 'static >, element_type) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSIndexedAccessType<'static>>() == 40usize);
        assert!(align_of::<TSIndexedAccessType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSIndexedAccessType < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(TSIndexedAccessType < 'static >,
            object_type) == 0usize); assert!(std::mem::offset_of!(TSIndexedAccessType <
            'static >, index_type) == 16usize);
        );
    }
    {
        assert!(size_of::<TSTupleType<'static>>() == 40usize);
        assert!(align_of::<TSTupleType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTupleType < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSTupleType < 'static >, element_types) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSNamedTupleMember<'static>>() == 56usize);
        assert!(align_of::<TSNamedTupleMember<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSNamedTupleMember < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(TSNamedTupleMember < 'static >,
            element_type) == 0usize); assert!(std::mem::offset_of!(TSNamedTupleMember <
            'static >, label) == 16usize);
            assert!(std::mem::offset_of!(TSNamedTupleMember < 'static >, optional) ==
            48usize);
        );
    }
    {
        assert!(size_of::<TSOptionalType<'static>>() == 24usize);
        assert!(align_of::<TSOptionalType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSOptionalType < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSOptionalType < 'static >, type_annotation) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSRestType<'static>>() == 24usize);
        assert!(align_of::<TSRestType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSRestType < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSRestType < 'static >, type_annotation) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSAnyKeyword>() == 8usize);
        assert!(align_of::<TSAnyKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSAnyKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSStringKeyword>() == 8usize);
        assert!(align_of::<TSStringKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSStringKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSBooleanKeyword>() == 8usize);
        assert!(align_of::<TSBooleanKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSBooleanKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSNumberKeyword>() == 8usize);
        assert!(align_of::<TSNumberKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSNumberKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSNeverKeyword>() == 8usize);
        assert!(align_of::<TSNeverKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSNeverKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSIntrinsicKeyword>() == 8usize);
        assert!(align_of::<TSIntrinsicKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSIntrinsicKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSUnknownKeyword>() == 8usize);
        assert!(align_of::<TSUnknownKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSUnknownKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSNullKeyword>() == 8usize);
        assert!(align_of::<TSNullKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSNullKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSUndefinedKeyword>() == 8usize);
        assert!(align_of::<TSUndefinedKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSUndefinedKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSVoidKeyword>() == 8usize);
        assert!(align_of::<TSVoidKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSVoidKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSSymbolKeyword>() == 8usize);
        assert!(align_of::<TSSymbolKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSSymbolKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSThisType>() == 8usize);
        assert!(align_of::<TSThisType>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSThisType, span) == 0usize););
    }
    {
        assert!(size_of::<TSObjectKeyword>() == 8usize);
        assert!(align_of::<TSObjectKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSObjectKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSBigIntKeyword>() == 8usize);
        assert!(align_of::<TSBigIntKeyword>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(TSBigIntKeyword, span) == 0usize););
    }
    {
        assert!(size_of::<TSTypeReference<'static>>() == 32usize);
        assert!(align_of::<TSTypeReference<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeReference < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSTypeReference < 'static >, type_name) ==
            0usize); assert!(std::mem::offset_of!(TSTypeReference < 'static >,
            type_parameters) == 24usize);
        );
    }
    {
        assert!(size_of::<TSQualifiedName<'static>>() == 48usize);
        assert!(align_of::<TSQualifiedName<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSQualifiedName < 'static >, span) == 40usize);
            assert!(std::mem::offset_of!(TSQualifiedName < 'static >, left) == 0usize);
            assert!(std::mem::offset_of!(TSQualifiedName < 'static >, right) == 16usize);
        );
    }
    {
        assert!(size_of::<TSTypeParameterInstantiation<'static>>() == 40usize);
        assert!(align_of::<TSTypeParameterInstantiation<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeParameterInstantiation < 'static >, span)
            == 32usize); assert!(std::mem::offset_of!(TSTypeParameterInstantiation <
            'static >, params) == 0usize);
        );
    }
    {
        assert!(size_of::<TSTypeParameter<'static>>() == 80usize);
        assert!(align_of::<TSTypeParameter<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeParameter < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(TSTypeParameter < 'static >, name) == 0usize);
            assert!(std::mem::offset_of!(TSTypeParameter < 'static >, constraint) ==
            32usize); assert!(std::mem::offset_of!(TSTypeParameter < 'static >, default)
            == 48usize); assert!(std::mem::offset_of!(TSTypeParameter < 'static >, r#in)
            == 72usize); assert!(std::mem::offset_of!(TSTypeParameter < 'static >, out)
            == 73usize); assert!(std::mem::offset_of!(TSTypeParameter < 'static >,
            r#const) == 74usize);
        );
    }
    {
        assert!(size_of::<TSTypeParameterDeclaration<'static>>() == 40usize);
        assert!(align_of::<TSTypeParameterDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeParameterDeclaration < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(TSTypeParameterDeclaration < 'static
            >, params) == 0usize);
        );
    }
    {
        assert!(size_of::<TSTypeAliasDeclaration<'static>>() == 72usize);
        assert!(align_of::<TSTypeAliasDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeAliasDeclaration < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(TSTypeAliasDeclaration < 'static >,
            id) == 16usize); assert!(std::mem::offset_of!(TSTypeAliasDeclaration <
            'static >, type_parameters) == 56usize);
            assert!(std::mem::offset_of!(TSTypeAliasDeclaration < 'static >,
            type_annotation) == 0usize);
            assert!(std::mem::offset_of!(TSTypeAliasDeclaration < 'static >, declare) ==
            68usize); assert!(std::mem::offset_of!(TSTypeAliasDeclaration < 'static >,
            scope_id) == 64usize);
        );
    }
    {
        assert!(size_of::<TSClassImplements<'static>>() == 32usize);
        assert!(align_of::<TSClassImplements<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSClassImplements < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSClassImplements < 'static >,
            expression) == 0usize); assert!(std::mem::offset_of!(TSClassImplements <
            'static >, type_parameters) == 24usize);
        );
    }
    {
        assert!(size_of::<TSInterfaceDeclaration<'static>>() == 96usize);
        assert!(align_of::<TSInterfaceDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSInterfaceDeclaration < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(TSInterfaceDeclaration < 'static >,
            id) == 0usize); assert!(std::mem::offset_of!(TSInterfaceDeclaration < 'static
            >, extends) == 48usize); assert!(std::mem::offset_of!(TSInterfaceDeclaration
            < 'static >, type_parameters) == 80usize);
            assert!(std::mem::offset_of!(TSInterfaceDeclaration < 'static >, body) ==
            32usize); assert!(std::mem::offset_of!(TSInterfaceDeclaration < 'static >,
            declare) == 92usize); assert!(std::mem::offset_of!(TSInterfaceDeclaration <
            'static >, scope_id) == 88usize);
        );
    }
    {
        assert!(size_of::<TSInterfaceBody<'static>>() == 40usize);
        assert!(align_of::<TSInterfaceBody<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSInterfaceBody < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSInterfaceBody < 'static >, body) == 0usize);
        );
    }
    {
        assert!(size_of::<TSPropertySignature<'static>>() == 40usize);
        assert!(align_of::<TSPropertySignature<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSPropertySignature < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSPropertySignature < 'static >,
            computed) == 32usize); assert!(std::mem::offset_of!(TSPropertySignature <
            'static >, optional) == 33usize);
            assert!(std::mem::offset_of!(TSPropertySignature < 'static >, readonly) ==
            34usize); assert!(std::mem::offset_of!(TSPropertySignature < 'static >, key)
            == 0usize); assert!(std::mem::offset_of!(TSPropertySignature < 'static >,
            type_annotation) == 24usize);
        );
    }
    {
        assert!(size_of::<TSIndexSignature<'static>>() == 56usize);
        assert!(align_of::<TSIndexSignature<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSIndexSignature < 'static >, span) == 40usize);
            assert!(std::mem::offset_of!(TSIndexSignature < 'static >, parameters) ==
            0usize); assert!(std::mem::offset_of!(TSIndexSignature < 'static >,
            type_annotation) == 32usize); assert!(std::mem::offset_of!(TSIndexSignature <
            'static >, readonly) == 48usize);
        );
    }
    {
        assert!(size_of::<TSCallSignatureDeclaration<'static>>() == 72usize);
        assert!(align_of::<TSCallSignatureDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSCallSignatureDeclaration < 'static >, span) ==
            8usize); assert!(std::mem::offset_of!(TSCallSignatureDeclaration < 'static >,
            this_param) == 16usize);
            assert!(std::mem::offset_of!(TSCallSignatureDeclaration < 'static >, params)
            == 0usize); assert!(std::mem::offset_of!(TSCallSignatureDeclaration < 'static
            >, return_type) == 56usize);
            assert!(std::mem::offset_of!(TSCallSignatureDeclaration < 'static >,
            type_parameters) == 64usize);
        );
    }
    {
        assert!(size_of::<TSMethodSignature<'static>>() == 96usize);
        assert!(align_of::<TSMethodSignature<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSMethodSignature < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSMethodSignature < 'static >, key) ==
            0usize); assert!(std::mem::offset_of!(TSMethodSignature < 'static >,
            computed) == 93usize); assert!(std::mem::offset_of!(TSMethodSignature <
            'static >, optional) == 94usize);
            assert!(std::mem::offset_of!(TSMethodSignature < 'static >, kind) ==
            92usize); assert!(std::mem::offset_of!(TSMethodSignature < 'static >,
            this_param) == 24usize); assert!(std::mem::offset_of!(TSMethodSignature <
            'static >, params) == 80usize);
            assert!(std::mem::offset_of!(TSMethodSignature < 'static >, return_type) ==
            64usize); assert!(std::mem::offset_of!(TSMethodSignature < 'static >,
            type_parameters) == 72usize); assert!(std::mem::offset_of!(TSMethodSignature
            < 'static >, scope_id) == 88usize);
        );
    }
    {
        assert!(size_of::<TSConstructSignatureDeclaration<'static>>() == 40usize);
        assert!(align_of::<TSConstructSignatureDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSConstructSignatureDeclaration < 'static >,
            span) == 8usize);
            assert!(std::mem::offset_of!(TSConstructSignatureDeclaration < 'static >,
            params) == 0usize);
            assert!(std::mem::offset_of!(TSConstructSignatureDeclaration < 'static >,
            return_type) == 16usize);
            assert!(std::mem::offset_of!(TSConstructSignatureDeclaration < 'static >,
            type_parameters) == 24usize);
            assert!(std::mem::offset_of!(TSConstructSignatureDeclaration < 'static >,
            scope_id) == 32usize);
        );
    }
    {
        assert!(size_of::<TSIndexSignatureName<'static>>() == 32usize);
        assert!(align_of::<TSIndexSignatureName<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSIndexSignatureName < 'static >, span) ==
            24usize); assert!(std::mem::offset_of!(TSIndexSignatureName < 'static >,
            name) == 0usize); assert!(std::mem::offset_of!(TSIndexSignatureName < 'static
            >, type_annotation) == 16usize);
        );
    }
    {
        assert!(size_of::<TSInterfaceHeritage<'static>>() == 32usize);
        assert!(align_of::<TSInterfaceHeritage<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSInterfaceHeritage < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSInterfaceHeritage < 'static >,
            expression) == 0usize); assert!(std::mem::offset_of!(TSInterfaceHeritage <
            'static >, type_parameters) == 24usize);
        );
    }
    {
        assert!(size_of::<TSTypePredicate<'static>>() == 40usize);
        assert!(align_of::<TSTypePredicate<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypePredicate < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSTypePredicate < 'static >, parameter_name) ==
            0usize); assert!(std::mem::offset_of!(TSTypePredicate < 'static >, asserts)
            == 32usize); assert!(std::mem::offset_of!(TSTypePredicate < 'static >,
            type_annotation) == 24usize);
        );
    }
    {
        assert!(size_of::<TSModuleDeclaration<'static>>() == 64usize);
        assert!(align_of::<TSModuleDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSModuleDeclaration < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(TSModuleDeclaration < 'static >, id)
            == 0usize); assert!(std::mem::offset_of!(TSModuleDeclaration < 'static >,
            body) == 32usize); assert!(std::mem::offset_of!(TSModuleDeclaration < 'static
            >, kind) == 61usize); assert!(std::mem::offset_of!(TSModuleDeclaration <
            'static >, declare) == 60usize);
            assert!(std::mem::offset_of!(TSModuleDeclaration < 'static >, scope_id) ==
            56usize);
        );
    }
    {
        assert!(size_of::<TSModuleBlock<'static>>() == 72usize);
        assert!(align_of::<TSModuleBlock<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSModuleBlock < 'static >, span) == 64usize);
            assert!(std::mem::offset_of!(TSModuleBlock < 'static >, directives) ==
            0usize); assert!(std::mem::offset_of!(TSModuleBlock < 'static >, body) ==
            32usize);
        );
    }
    {
        assert!(size_of::<TSTypeLiteral<'static>>() == 40usize);
        assert!(align_of::<TSTypeLiteral<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeLiteral < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSTypeLiteral < 'static >, members) == 0usize);
        );
    }
    {
        assert!(size_of::<TSInferType<'static>>() == 16usize);
        assert!(align_of::<TSInferType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSInferType < 'static >, span) == 8usize);
            assert!(std::mem::offset_of!(TSInferType < 'static >, type_parameter) ==
            0usize);
        );
    }
    {
        assert!(size_of::<TSTypeQuery<'static>>() == 32usize);
        assert!(align_of::<TSTypeQuery<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeQuery < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSTypeQuery < 'static >, expr_name) == 0usize);
            assert!(std::mem::offset_of!(TSTypeQuery < 'static >, type_parameters) ==
            24usize);
        );
    }
    {
        assert!(size_of::<TSImportType<'static>>() == 96usize);
        assert!(align_of::<TSImportType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSImportType < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSImportType < 'static >, is_type_of) ==
            88usize); assert!(std::mem::offset_of!(TSImportType < 'static >, parameter)
            == 16usize); assert!(std::mem::offset_of!(TSImportType < 'static >,
            qualifier) == 0usize); assert!(std::mem::offset_of!(TSImportType < 'static >,
            attributes) == 40usize); assert!(std::mem::offset_of!(TSImportType < 'static
            >, type_parameters) == 80usize);
        );
    }
    {
        assert!(size_of::<TSImportAttributes<'static>>() == 40usize);
        assert!(align_of::<TSImportAttributes<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSImportAttributes < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(TSImportAttributes < 'static >,
            elements) == 0usize);
        );
    }
    {
        assert!(size_of::<TSImportAttribute<'static>>() == 56usize);
        assert!(align_of::<TSImportAttribute<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSImportAttribute < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(TSImportAttribute < 'static >, name)
            == 0usize); assert!(std::mem::offset_of!(TSImportAttribute < 'static >,
            value) == 32usize);
        );
    }
    {
        assert!(size_of::<TSFunctionType<'static>>() == 72usize);
        assert!(align_of::<TSFunctionType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSFunctionType < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(TSFunctionType < 'static >, this_param) ==
            24usize); assert!(std::mem::offset_of!(TSFunctionType < 'static >, params) ==
            0usize); assert!(std::mem::offset_of!(TSFunctionType < 'static >,
            return_type) == 8usize); assert!(std::mem::offset_of!(TSFunctionType <
            'static >, type_parameters) == 64usize);
        );
    }
    {
        assert!(size_of::<TSConstructorType<'static>>() == 40usize);
        assert!(align_of::<TSConstructorType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSConstructorType < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSConstructorType < 'static >,
            r#abstract) == 32usize); assert!(std::mem::offset_of!(TSConstructorType <
            'static >, params) == 0usize); assert!(std::mem::offset_of!(TSConstructorType
            < 'static >, return_type) == 8usize);
            assert!(std::mem::offset_of!(TSConstructorType < 'static >, type_parameters)
            == 24usize);
        );
    }
    {
        assert!(size_of::<TSMappedType<'static>>() == 56usize);
        assert!(align_of::<TSMappedType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSMappedType < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSMappedType < 'static >, type_parameter) ==
            40usize); assert!(std::mem::offset_of!(TSMappedType < 'static >, name_type)
            == 0usize); assert!(std::mem::offset_of!(TSMappedType < 'static >,
            type_annotation) == 16usize); assert!(std::mem::offset_of!(TSMappedType <
            'static >, optional) == 52usize); assert!(std::mem::offset_of!(TSMappedType <
            'static >, readonly) == 53usize); assert!(std::mem::offset_of!(TSMappedType <
            'static >, scope_id) == 48usize);
        );
    }
    {
        assert!(size_of::<TSTemplateLiteralType<'static>>() == 72usize);
        assert!(align_of::<TSTemplateLiteralType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTemplateLiteralType < 'static >, span) ==
            64usize); assert!(std::mem::offset_of!(TSTemplateLiteralType < 'static >,
            quasis) == 0usize); assert!(std::mem::offset_of!(TSTemplateLiteralType <
            'static >, types) == 32usize);
        );
    }
    {
        assert!(size_of::<TSAsExpression<'static>>() == 40usize);
        assert!(align_of::<TSAsExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSAsExpression < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSAsExpression < 'static >, expression) ==
            16usize); assert!(std::mem::offset_of!(TSAsExpression < 'static >,
            type_annotation) == 0usize);
        );
    }
    {
        assert!(size_of::<TSSatisfiesExpression<'static>>() == 40usize);
        assert!(align_of::<TSSatisfiesExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSSatisfiesExpression < 'static >, span) ==
            32usize); assert!(std::mem::offset_of!(TSSatisfiesExpression < 'static >,
            expression) == 16usize); assert!(std::mem::offset_of!(TSSatisfiesExpression <
            'static >, type_annotation) == 0usize);
        );
    }
    {
        assert!(size_of::<TSTypeAssertion<'static>>() == 40usize);
        assert!(align_of::<TSTypeAssertion<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSTypeAssertion < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(TSTypeAssertion < 'static >, expression) ==
            16usize); assert!(std::mem::offset_of!(TSTypeAssertion < 'static >,
            type_annotation) == 0usize);
        );
    }
    {
        assert!(size_of::<TSImportEqualsDeclaration<'static>>() == 64usize);
        assert!(align_of::<TSImportEqualsDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSImportEqualsDeclaration < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(TSImportEqualsDeclaration < 'static >,
            id) == 16usize); assert!(std::mem::offset_of!(TSImportEqualsDeclaration <
            'static >, module_reference) == 0usize);
            assert!(std::mem::offset_of!(TSImportEqualsDeclaration < 'static >,
            import_kind) == 56usize);
        );
    }
    {
        assert!(size_of::<TSExternalModuleReference<'static>>() == 32usize);
        assert!(align_of::<TSExternalModuleReference<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSExternalModuleReference < 'static >, span) ==
            24usize); assert!(std::mem::offset_of!(TSExternalModuleReference < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<TSNonNullExpression<'static>>() == 24usize);
        assert!(align_of::<TSNonNullExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSNonNullExpression < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSNonNullExpression < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<Decorator<'static>>() == 24usize);
        assert!(align_of::<Decorator<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(Decorator < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(Decorator < 'static >, expression) == 0usize);
        );
    }
    {
        assert!(size_of::<TSExportAssignment<'static>>() == 24usize);
        assert!(align_of::<TSExportAssignment<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSExportAssignment < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(TSExportAssignment < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<TSNamespaceExportDeclaration<'static>>() == 32usize);
        assert!(align_of::<TSNamespaceExportDeclaration<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSNamespaceExportDeclaration < 'static >, span)
            == 24usize); assert!(std::mem::offset_of!(TSNamespaceExportDeclaration <
            'static >, id) == 0usize);
        );
    }
    {
        assert!(size_of::<TSInstantiationExpression<'static>>() == 32usize);
        assert!(align_of::<TSInstantiationExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(TSInstantiationExpression < 'static >, span) ==
            24usize); assert!(std::mem::offset_of!(TSInstantiationExpression < 'static >,
            expression) == 0usize);
            assert!(std::mem::offset_of!(TSInstantiationExpression < 'static >,
            type_parameters) == 16usize);
        );
    }
    {
        assert!(size_of::<JSDocNullableType<'static>>() == 32usize);
        assert!(align_of::<JSDocNullableType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSDocNullableType < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(JSDocNullableType < 'static >,
            type_annotation) == 0usize); assert!(std::mem::offset_of!(JSDocNullableType <
            'static >, postfix) == 24usize);
        );
    }
    {
        assert!(size_of::<JSDocNonNullableType<'static>>() == 32usize);
        assert!(align_of::<JSDocNonNullableType<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSDocNonNullableType < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(JSDocNonNullableType < 'static >,
            type_annotation) == 0usize);
            assert!(std::mem::offset_of!(JSDocNonNullableType < 'static >, postfix) ==
            24usize);
        );
    }
    {
        assert!(size_of::<JSDocUnknownType>() == 8usize);
        assert!(align_of::<JSDocUnknownType>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(JSDocUnknownType, span) == 0usize););
    }
    {
        assert!(size_of::<JSXElement<'static>>() == 56usize);
        assert!(align_of::<JSXElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXElement < 'static >, span) == 40usize);
            assert!(std::mem::offset_of!(JSXElement < 'static >, opening_element) ==
            0usize); assert!(std::mem::offset_of!(JSXElement < 'static >,
            closing_element) == 48usize); assert!(std::mem::offset_of!(JSXElement <
            'static >, children) == 8usize);
        );
    }
    {
        assert!(size_of::<JSXOpeningElement<'static>>() == 72usize);
        assert!(align_of::<JSXOpeningElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXOpeningElement < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(JSXOpeningElement < 'static >,
            self_closing) == 64usize); assert!(std::mem::offset_of!(JSXOpeningElement <
            'static >, name) == 0usize); assert!(std::mem::offset_of!(JSXOpeningElement <
            'static >, attributes) == 16usize);
            assert!(std::mem::offset_of!(JSXOpeningElement < 'static >, type_parameters)
            == 56usize);
        );
    }
    {
        assert!(size_of::<JSXClosingElement<'static>>() == 24usize);
        assert!(align_of::<JSXClosingElement<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXClosingElement < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(JSXClosingElement < 'static >, name)
            == 0usize);
        );
    }
    {
        assert!(size_of::<JSXFragment<'static>>() == 56usize);
        assert!(align_of::<JSXFragment<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXFragment < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(JSXFragment < 'static >, opening_fragment) ==
            40usize); assert!(std::mem::offset_of!(JSXFragment < 'static >,
            closing_fragment) == 48usize); assert!(std::mem::offset_of!(JSXFragment <
            'static >, children) == 0usize);
        );
    }
    {
        assert!(size_of::<JSXOpeningFragment>() == 8usize);
        assert!(align_of::<JSXOpeningFragment>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(JSXOpeningFragment, span) == 0usize););
    }
    {
        assert!(size_of::<JSXClosingFragment>() == 8usize);
        assert!(align_of::<JSXClosingFragment>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(JSXClosingFragment, span) == 0usize););
    }
    {
        assert!(size_of::<JSXNamespacedName<'static>>() == 56usize);
        assert!(align_of::<JSXNamespacedName<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXNamespacedName < 'static >, span) ==
            48usize); assert!(std::mem::offset_of!(JSXNamespacedName < 'static >,
            namespace) == 0usize); assert!(std::mem::offset_of!(JSXNamespacedName <
            'static >, property) == 24usize);
        );
    }
    {
        assert!(size_of::<JSXMemberExpression<'static>>() == 48usize);
        assert!(align_of::<JSXMemberExpression<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXMemberExpression < 'static >, span) ==
            40usize); assert!(std::mem::offset_of!(JSXMemberExpression < 'static >,
            object) == 0usize); assert!(std::mem::offset_of!(JSXMemberExpression <
            'static >, property) == 16usize);
        );
    }
    {
        assert!(size_of::<JSXExpressionContainer<'static>>() == 24usize);
        assert!(align_of::<JSXExpressionContainer<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXExpressionContainer < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(JSXExpressionContainer < 'static >,
            expression) == 0usize);
        );
    }
    {
        assert!(size_of::<JSXEmptyExpression>() == 8usize);
        assert!(align_of::<JSXEmptyExpression>() == 4usize);
        wrap!(assert!(std::mem::offset_of!(JSXEmptyExpression, span) == 0usize););
    }
    {
        assert!(size_of::<JSXAttribute<'static>>() == 40usize);
        assert!(align_of::<JSXAttribute<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXAttribute < 'static >, span) == 32usize);
            assert!(std::mem::offset_of!(JSXAttribute < 'static >, name) == 0usize);
            assert!(std::mem::offset_of!(JSXAttribute < 'static >, value) == 16usize);
        );
    }
    {
        assert!(size_of::<JSXSpreadAttribute<'static>>() == 24usize);
        assert!(align_of::<JSXSpreadAttribute<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXSpreadAttribute < 'static >, span) ==
            16usize); assert!(std::mem::offset_of!(JSXSpreadAttribute < 'static >,
            argument) == 0usize);
        );
    }
    {
        assert!(size_of::<JSXIdentifier<'static>>() == 24usize);
        assert!(align_of::<JSXIdentifier<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXIdentifier < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(JSXIdentifier < 'static >, name) == 0usize);
        );
    }
    {
        assert!(size_of::<JSXSpreadChild<'static>>() == 24usize);
        assert!(align_of::<JSXSpreadChild<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXSpreadChild < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(JSXSpreadChild < 'static >, expression) ==
            0usize);
        );
    }
    {
        assert!(size_of::<JSXText<'static>>() == 24usize);
        assert!(align_of::<JSXText<'static>>() == 8usize);
        wrap!(
            assert!(std::mem::offset_of!(JSXText < 'static >, span) == 16usize);
            assert!(std::mem::offset_of!(JSXText < 'static >, value) == 0usize);
        );
    }
};
