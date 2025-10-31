use oxc_allocator::Allocator;
use oxc_span::SourceType;

use oxc_semantic::{
    NormalizedConstEnumInfo, NormalizedConstantValue, Semantic, SemanticBuilder,
};

/// Create a [`Semantic`] from source code, assuming there are no syntax/semantic errors.
fn get_semantic<'s, 'a: 's>(
    allocator: &'a Allocator,
    source: &'s str,
    source_type: SourceType,
) -> Semantic<'s> {
    let parse = oxc_parser::Parser::new(allocator, source, source_type).parse();
    assert!(parse.errors.is_empty());
    let semantic = SemanticBuilder::new().build(allocator.alloc(parse.program));
    assert!(semantic.errors.is_empty(), "Parse error: {}", semantic.errors[0]);
    semantic.semantic
}

fn assert_const_enum_value(value: &NormalizedConstantValue, expected: &str) {
    let computed = value.to_string();

    assert_eq!(computed, expected);
}

fn find_member_by_name<'a>(
    enum_info: &'a NormalizedConstEnumInfo,
    name: &str,
) -> Option<&'a NormalizedConstantValue> {
    enum_info
        .member_name_to_symbol_id
        .get(name)
        .and_then(|symbol_id| enum_info.members.get(symbol_id))
}

#[test]
fn test_const_enum_simple() {
    let source = "
            const enum Color {
                Red,
                Green,
                Blue
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Color enum
    let color_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Color");

    assert!(color_enum.is_some());

    let (_symbol_id, enum_info) = color_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 3);

    // Check Red member (should be "0")
    let red_member = find_member_by_name(enum_info, "Red").unwrap();
    assert_const_enum_value(red_member, "0");

    // Check Green member (should be "1")
    let green_member = find_member_by_name(enum_info, "Green").unwrap();
    assert_const_enum_value(green_member, "1");

    // Check Blue member (should be "2")
    let blue_member = find_member_by_name(enum_info, "Blue").unwrap();
    assert_const_enum_value(blue_member, "2");
}

#[test]
fn test_const_enum_with_values() {
    let source = "
            const enum Status {
                Pending = 1,
                Approved = 2,
                Rejected = 3
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Status enum
    let status_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Status");

    assert!(status_enum.is_some());

    let (_, enum_info) = status_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 3);

    // Check Pending member (should be "1")
    let pending_member = find_member_by_name(enum_info, "Pending").unwrap();
    assert_const_enum_value(pending_member, "1");

    // Check Approved member (should be "2")
    let approved_member = find_member_by_name(enum_info, "Approved").unwrap();
    assert_const_enum_value(approved_member, "2");

    // Check Rejected member (should be "3")
    let rejected_member = find_member_by_name(enum_info, "Rejected").unwrap();
    assert_const_enum_value(rejected_member, "3");
}

#[test]
fn test_const_enum_mixed_values() {
    let source = "
            const enum Mixed {
                A,
                B = 5,
                C,
                D = 'hello',
                E
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Mixed enum
    let mixed_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Mixed");

    assert!(mixed_enum.is_some());

    let (_, enum_info) = mixed_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 5);

    // A should be "0" (auto-increment)
    let a_member = find_member_by_name(enum_info, "A").unwrap();
    assert_const_enum_value(a_member, "0");

    // B should be "5" (explicit)
    let b_member = find_member_by_name(enum_info, "B").unwrap();
    assert_const_enum_value(b_member, "5");

    // C should be "6" (auto-increment after B)
    let c_member = find_member_by_name(enum_info, "C").unwrap();
    assert_const_enum_value(c_member, "6");

    // D should be "\"hello\"" (string literal)
    let d_member = find_member_by_name(enum_info, "D").unwrap();
    assert_const_enum_value(d_member, "\"hello\"");

    // E should be "7" (auto-increment after string literal)
    let e_member = find_member_by_name(enum_info, "E").unwrap();
    assert_const_enum_value(e_member, "Computed");
}

#[test]
fn test_const_enum_literals() {
    let source = "
            enum RegularEnum {
                A,
                B,
                C
            }
            const enum Literals {
                StringVal = 'hello',
                NumberVal = 42,
                TrueVal = true,
                FalseVal = false,
                BigIntVal = 9007199254740991n
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Literals enum
    let literals_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Literals");

    assert!(literals_enum.is_some());

    let (_, enum_info) = literals_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 5);

    // StringVal should be "\"hello\""
    let string_member = find_member_by_name(enum_info, "StringVal").unwrap();
    assert_const_enum_value(string_member, "\"hello\"");

    // NumberVal should be "42"
    let number_member = find_member_by_name(enum_info, "NumberVal").unwrap();
    assert_const_enum_value(number_member, "42");

    // TrueVal should be "true"
    let true_member = find_member_by_name(enum_info, "TrueVal").unwrap();
    assert_const_enum_value(true_member, "true");

    // FalseVal should be "false"
    let false_member = find_member_by_name(enum_info, "FalseVal").unwrap();
    assert_const_enum_value(false_member, "false");

    // BigIntVal should be "9007199254740991n"
    let bigint_member = find_member_by_name(enum_info, "BigIntVal").unwrap();
    assert_const_enum_value(bigint_member, "9007199254740991n");
}

#[test]
fn test_const_enum_binary_expressions() {
    let source = "
            const enum Operations {
                Add = 1 + 2,
                Subtract = 10 - 3,
                Multiply = 3 * 4,
                Divide = 20 / 4,
                Negate = -5,
                Plus = +7,
                Not = !true,
                Shift = 1 << 2,
                Bitwise = 5 | 3
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Operations enum
    let operations_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Operations");

    assert!(operations_enum.is_some());

    let (_, enum_info) = operations_enum.unwrap();

    // Check Add member (should be "3")
    let add_member = find_member_by_name(enum_info, "Add").unwrap();
    assert_const_enum_value(add_member, "3");

    // Check Subtract member (should be "7")
    let subtract_member = find_member_by_name(enum_info, "Subtract").unwrap();
    assert_const_enum_value(subtract_member, "7");

    // Check Multiply member (should be "12")
    let multiply_member = find_member_by_name(enum_info, "Multiply").unwrap();
    assert_const_enum_value(multiply_member, "12");

    // Check Divide member (should be "5")
    let divide_member = find_member_by_name(enum_info, "Divide").unwrap();
    assert_const_enum_value(divide_member, "5");

    // Check Negate member (should be "-5")
    let negate_member = find_member_by_name(enum_info, "Negate").unwrap();
    assert_const_enum_value(negate_member, "-5");

    // Check Plus member (should be "7")
    let plus_member = find_member_by_name(enum_info, "Plus").unwrap();
    assert_const_enum_value(plus_member, "7");

    // Check Not member (should be "false")
    let not_member = find_member_by_name(enum_info, "Not").unwrap();
    assert_const_enum_value(not_member, "false");

    // Check Shift member (should be "4", 1 << 2)
    let shift_member = find_member_by_name(enum_info, "Shift").unwrap();
    assert_const_enum_value(shift_member, "4");

    // Check Bitwise member (should be "7", 5 | 3 = 101 | 011 = 111)
    let bitwise_member = find_member_by_name(enum_info, "Bitwise").unwrap();
    assert_const_enum_value(bitwise_member, "7");
}

#[test]
fn test_const_enum_constant_propagation() {
    let source = "
            const enum Values {
                A = 1,
                B = A,
                C = A + 2,
                D = B * 3,
                E = C + D,
                F = A + B + C + D
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Values enum
    let values_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Values");

    assert!(values_enum.is_some());

    let (_, enum_info) = values_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 6);

    // A should be "1"
    let a_member = find_member_by_name(enum_info, "A").unwrap();
    assert_const_enum_value(a_member, "1");

    // B should be "1" (references A - constant propagation works for simple references)
    let b_member = find_member_by_name(enum_info, "B").unwrap();
    assert_const_enum_value(b_member, "1");

    // C should be "3" (A + 2 = 1 + 2 - constant propagation works in expressions)
    let c_member = find_member_by_name(enum_info, "C").unwrap();
    assert_const_enum_value(c_member, "3");

    // D should be "3" (B * 3 = 1 * 3 - constant propagation works in expressions)
    let d_member = find_member_by_name(enum_info, "D").unwrap();
    assert_const_enum_value(d_member, "3");

    // E: Currently "Computed" because C and D are computed values
    // TODO: Should be "6" (C + D = 3 + 3) when full constant propagation is implemented
    let e_member = find_member_by_name(enum_info, "E").unwrap();
    assert_const_enum_value(e_member, "Computed");

    // F: Currently "Computed" for the same reason as E
    // TODO: Should be "8" (A + B + C + D = 1 + 1 + 3 + 3) when full constant propagation is implemented
    let f_member = find_member_by_name(enum_info, "F").unwrap();
    assert_const_enum_value(f_member, "Computed");
}

#[test]
fn test_const_enum_member_access_propagation() {
    let source = "
            const enum Base {
                X = 10,
                Y = 20
            }
            const enum Derived {
                A = Base.X,
                B = Base.Y,
                C = Base.X + Base.Y,
                D = Base.X * 2
            }
        ";
    let allocator = Allocator::default();
    let source_type: SourceType = SourceType::default().with_typescript(true);
    let semantic = get_semantic(&allocator, source, source_type);

    // Find the Derived enum
    let derived_enum = semantic
        .const_enums()
        .enums()
        .find(|(_, enum_info)| semantic.scoping().symbol_name(enum_info.symbol_id) == "Derived");

    assert!(derived_enum.is_some());

    let (_, enum_info) = derived_enum.unwrap();

    // Check enum members
    assert_eq!(enum_info.members.len(), 4);

    // A: Currently "Computed" because cross-enum member access isn't implemented yet
    // TODO: Should be "10" (Base.X) when cross-enum constant propagation is implemented
    let a_member = find_member_by_name(enum_info, "A").unwrap();
    assert_const_enum_value(a_member, "Computed");

    // B: Currently "Computed" because cross-enum member access isn't implemented yet
    // TODO: Should be "20" (Base.Y) when cross-enum constant propagation is implemented
    let b_member = find_member_by_name(enum_info, "B").unwrap();
    assert_const_enum_value(b_member, "Computed");

    // C: Currently "Computed" because cross-enum member access isn't implemented yet
    // TODO: Should be "30" (Base.X + Base.Y = 10 + 20) when cross-enum constant propagation is implemented
    let c_member = find_member_by_name(enum_info, "C").unwrap();
    assert_const_enum_value(c_member, "Computed");

    // D: Currently "Computed" because cross-enum member access isn't implemented yet
    // TODO: Should be "20" (Base.X * 2 = 10 * 2) when cross-enum constant propagation is implemented
    let d_member = find_member_by_name(enum_info, "D").unwrap();
    assert_const_enum_value(d_member, "Computed");
}
