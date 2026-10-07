use std::cell::Cell;

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{
    AstKind, CommentAttachment, CommentPlacement,
    ast::{Program, Statement},
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::{NodeId, SemanticBuilder};
use oxc_span::SourceType;

fn attach_to_statement(program: &mut Program<'_>, comment: usize, statement: usize) {
    program.comments[comment].attachment = Some(CommentAttachment {
        node_id: Cell::new(program.body[statement].node_id()),
        placement: CommentPlacement::Leading,
    });
}

#[test]
fn ordinary_build_preserves_sparse_and_reordered_owners() {
    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let mut program = Parser::new(
            &allocator,
            "/* first */ first(); /* second */ second();",
            SourceType::mjs(),
        )
        .parse()
        .program;
        let first = NodeId::new(NodeId::MAX_INDEX - 1);
        let second = NodeId::new(100_003);
        for (statement, id) in program.body.iter().zip([first, second]) {
            let Statement::ExpressionStatement(statement) = statement else { unreachable!() };
            statement.node_id.set(id);
        }
        attach_to_statement(&mut program, 0, 0);
        attach_to_statement(&mut program, 1, 1);
        program.comments[1].attachment.as_mut().unwrap().placement = CommentPlacement::Trailing;
        program.body.swap(0, 1);

        for _ in 0..2 {
            let result = SemanticBuilder::new().with_build_nodes(full_nodes).build(&program);
            assert!(result.diagnostics.is_empty());
            if full_nodes {
                for comment in result.semantic.comments() {
                    let owner = comment.attachment.as_ref().unwrap().node_id.get();
                    assert_eq!(result.semantic.nodes().kind(owner).node_id(), owner);
                }
                assert_eq!(
                    result.semantic.nodes().program().comments.as_ptr(),
                    result.semantic.comments().as_ptr(),
                );
            }
            drop(result);
            assert_eq!(
                program.comments[0].attachment.as_ref().unwrap().node_id.get(),
                program.body[1].node_id()
            );
            assert_eq!(
                program.comments[1].attachment.as_ref().unwrap().node_id.get(),
                program.body[0].node_id()
            );
            assert_eq!(
                program.comments[0].attachment.as_ref().unwrap().placement,
                CommentPlacement::Leading
            );
            assert_eq!(
                program.comments[1].attachment.as_ref().unwrap().placement,
                CommentPlacement::Trailing
            );
        }
    }
}

#[test]
fn removed_owners_stay_orphaned_and_unassigned_comments_stay_unassigned() {
    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let mut program = Parser::new(
            &allocator,
            "/* removed */ first(); /* unassigned */ second();",
            SourceType::mjs(),
        )
        .parse()
        .program;
        attach_to_statement(&mut program, 0, 0);
        program.comments[1].attachment = None;
        program.body.remove(0);
        for _ in 0..2 {
            SemanticBuilder::new().with_build_nodes(full_nodes).build(&program);
            assert_eq!(
                program.comments[0].attachment.as_ref().unwrap().node_id.get(),
                NodeId::ORPHANED
            );
            assert_eq!(program.comments[1].attachment, None);
        }
    }
}

#[test]
fn synthetic_nodes_do_not_claim_program_comments() {
    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let mut program =
            Parser::new(&allocator, "/* file */ first();", SourceType::mjs()).parse().program;
        program.comments[0].attachment = Some(CommentAttachment {
            node_id: Cell::new(NodeId::ROOT),
            placement: CommentPlacement::Dangling,
        });
        let synthetic = program.body[0].clone_in(&allocator);
        assert_eq!(synthetic.node_id(), NodeId::DUMMY);
        program.body.insert(0, synthetic);
        SemanticBuilder::new().with_build_nodes(full_nodes).build(&program);
        assert_eq!(program.comments[0].attachment.as_ref().unwrap().node_id.get(), NodeId::ROOT);
    }
}

#[test]
fn duplicate_old_ids_claim_the_first_surviving_owner() {
    let allocator = Allocator::default();
    let mut program =
        Parser::new(&allocator, "/* owner */ first();", SourceType::mjs()).parse().program;
    attach_to_statement(&mut program, 0, 0);
    let duplicate = program.body[0].clone_in_with_semantic_ids(&allocator);
    program.body.insert(0, duplicate);
    SemanticBuilder::new().build(&program);
    assert_eq!(
        program.comments[0].attachment.as_ref().unwrap().node_id.get(),
        program.body[0].node_id()
    );
    assert_ne!(program.body[0].node_id(), program.body[1].node_id());
}

#[test]
fn reordered_duplicates_preserve_grouped_comments_with_a_removed_owner() {
    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let mut program = Parser::new(
            &allocator,
            "/* first */ first(); /* second */ second(); /* removed */ third();",
            SourceType::mjs(),
        )
        .parse()
        .program;
        for (statement, id) in program.body.iter().zip([100, 10, 50]) {
            let Statement::ExpressionStatement(statement) = statement else { unreachable!() };
            statement.node_id.set(NodeId::new(id));
        }
        attach_to_statement(&mut program, 0, 1);
        attach_to_statement(&mut program, 1, 1);
        attach_to_statement(&mut program, 2, 2);
        program.body.remove(2);
        let duplicate = program.body[1].clone_in_with_semantic_ids(&allocator);
        program.body.push(duplicate);

        for _ in 0..2 {
            SemanticBuilder::new().with_build_nodes(full_nodes).build(&program);
            for comment in &program.comments[..2] {
                assert_eq!(
                    comment.attachment.as_ref().unwrap().node_id.get(),
                    program.body[1].node_id()
                );
            }
            assert_ne!(program.body[1].node_id(), program.body[2].node_id());
            assert_eq!(
                program.comments[2].attachment.as_ref().unwrap().node_id.get(),
                NodeId::ORPHANED
            );
        }
    }
}

#[test]
fn clones_without_node_ids_clear_comment_ownership() {
    let allocator = Allocator::default();
    let mut program =
        Parser::new(&allocator, "/* owner */ first();", SourceType::mjs()).parse().program;
    attach_to_statement(&mut program, 0, 0);
    let cloned = program.clone_in(&allocator);
    SemanticBuilder::new().build(&cloned);
    assert!(cloned.comments.iter().all(|comment| comment.attachment.is_none()));
}

#[test]
fn shared_attachment_reference_observes_remapping() {
    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let mut program =
            Parser::new(&allocator, "/* owner */ first();", SourceType::mjs()).parse().program;
        attach_to_statement(&mut program, 0, 0);
        let attachment = program.comments[0].attachment.as_ref().unwrap();
        let old_id = NodeId::new(100_003);
        let Statement::ExpressionStatement(statement) = &program.body[0] else { unreachable!() };
        statement.node_id.set(old_id);
        attachment.node_id.set(old_id);
        SemanticBuilder::new().with_build_nodes(full_nodes).build(&program);
        assert_ne!(attachment.node_id.get(), old_id);
        assert_eq!(attachment.node_id.get(), program.body[0].node_id());
        assert_eq!(attachment.placement, CommentPlacement::Leading);
    }
}

#[test]
fn construction_order_preserves_nested_and_later_owners() {
    #[derive(Default)]
    struct Nodes<'a>(Vec<AstKind<'a>>);

    impl<'a> Visit<'a> for Nodes<'a> {
        fn enter_node(&mut self, kind: AstKind<'a>) {
            self.0.push(kind);
        }
    }

    for full_nodes in [false, true] {
        let allocator = Allocator::default();
        let program = Parser::new(
            &allocator,
            "/* function */ function first() { before(); /* nested */ nested(); } \
             /* later */ later();",
            SourceType::mjs(),
        )
        .parse()
        .program;
        assert_eq!(program.comments.len(), 3);
        let mut nodes = Nodes::default();
        nodes.visit_program(&program);
        let owners = program
            .comments
            .iter()
            .map(|comment| {
                let attachment = comment.attachment.as_ref().unwrap();
                let kind = nodes
                    .0
                    .iter()
                    .copied()
                    .find(|kind| kind.node_id() == attachment.node_id.get())
                    .unwrap();
                (attachment, kind)
            })
            .collect::<Vec<_>>();

        for _ in 0..2 {
            SemanticBuilder::new_compiler().with_build_nodes(full_nodes).build(&program);
            for (attachment, kind) in &owners {
                assert_eq!(attachment.node_id.get(), kind.node_id());
            }
        }
    }
}
