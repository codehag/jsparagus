use crate::context_stack::{
    BindingInfo, BindingKind, BindingsIndex, BreakOrContinueIndex, ContextMetadata, LabelIndex,
    LabelInfo, LabelKind,
};
use crate::declaration_kind::DeclarationKind;
use crate::early_errors::*;
use crate::error::{BoxedParseError, ParseError, Result};
use crate::Token;
use ast::{
    arena,
    source_atom_set::{SourceAtomSet, SourceAtomSetIndex},
    types::*,
};
use std::cell::RefCell;
use std::rc::Rc;

/// called From EarlyErrorChecker::check_labelled_statement, and not used by
/// the struct implementing the trait. This means that
/// LabelledStatementEarlyErrorsContext is allocated inside this trait.
fn check_labelled_continue_to_non_loop<'alloc>(
    context_metadata_mut: &mut ContextMetadata,
    context: LabelledStatementEarlyErrorsContext,
    index: BreakOrContinueIndex,
) -> Result<'alloc, ()> {
    for info in context_metadata_mut.breaks_and_continues_from(index) {
        context.check_labelled_continue_to_non_loop(info)?;
    }

    Ok(())
}

/// called From EarlyErrorChecker::check_script_bindings
/// EarlyErrorChecker::check_module_bindings,
/// EarlyErrorChecker::check_function_bindings, and not used by
/// the struct implementing the trait. This means that
/// the contexts associated with those methods are allocated inside this trait.
fn check_unhandled_break_or_continue<'alloc, T>(
    context_metadata_mut: &mut ContextMetadata,
    context: T,
    offset: usize,
) -> Result<'alloc, ()>
where
    T: ControlEarlyErrorsContext,
{
    let index = context_metadata_mut.find_first_break_or_continue(offset);
    if let Some(info) = context_metadata_mut.find_break_or_continue_at(index) {
        context.on_unhandled_break_or_continue(info)?;
    }

    Ok(())
}

/// https://tc39.es/ecma262/#sec-islabelledfunction
/// Static Semantics: IsLabelledFunction ( stmt )
fn is_labelled_function(context_metadata: &ContextMetadata, statement_start_offset: usize) -> bool {
    // Step 1. If stmt is not a LabelledStatement , return false.
    if let Some(index) = context_metadata.find_label_index_at_offset(statement_start_offset) {
        // Step 2. Let item be the LabelledItem of stmt.
        for label in context_metadata.labels_from(index) {
            match label.kind {
                // Step 3. If item is LabelledItem : FunctionDeclaration,
                // return true.
                LabelKind::Function => {
                    return true;
                }
                // Step 4. Let subStmt be the Statement of item.
                // Step 5. Return IsLabelledFunction(subStmt).
                LabelKind::LabelledLabel => continue,
                _ => break,
            }
        }
    }

    false
}

/// Declare bindings in context_metadata to script-or-function-like context,
/// where function declarations are body-level. This method is an internal
/// helper for EarlyErrorChecker
fn declare_script_or_function<'alloc, T>(
    context_metadata: &ContextMetadata,
    atoms: &Rc<RefCell<SourceAtomSet<'alloc>>>,
    context: &mut T,
    index: BindingsIndex,
) -> Result<'alloc, ()>
where
    T: LexicalEarlyErrorsContext + VarEarlyErrorsContext,
{
    for info in context_metadata.bindings_from(index) {
        match info.kind {
            BindingKind::Var => {
                context.declare_var(
                    info.name,
                    DeclarationKind::Var,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Function | BindingKind::AsyncOrGenerator => {
                context.declare_var(
                    info.name,
                    DeclarationKind::BodyLevelFunction,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Let => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Let,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Const => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Const,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Class => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Class,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            _ => {
                panic!("Unexpected binding found {:?}", info);
            }
        }
    }

    Ok(())
}

/// Declare bindings to Block-like context, where function declarations
/// are lexical.  This method is an internal helper for EarlyErrorChecker
fn declare_block<'alloc, T>(
    context_metadata: &ContextMetadata,
    atoms: &Rc<RefCell<SourceAtomSet<'alloc>>>,
    context: &mut T,
    index: BindingsIndex,
) -> Result<'alloc, ()>
where
    T: LexicalEarlyErrorsContext + VarEarlyErrorsContext,
{
    for info in context_metadata.bindings_from(index) {
        match info.kind {
            BindingKind::Var => {
                context.declare_var(
                    info.name,
                    DeclarationKind::Var,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Function => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::LexicalFunction,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::AsyncOrGenerator => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::LexicalAsyncOrGenerator,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Let => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Let,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Const => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Const,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Class => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Class,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            _ => {
                panic!("Unexpected binding found {:?}", info);
            }
        }
    }

    Ok(())
}

/// Declare bindings to the parameter of function or catch.
/// This method is an internal helper for EarlyErrorChecker
fn declare_param<'alloc, T>(
    context_metadata: &ContextMetadata,
    atoms: &Rc<RefCell<SourceAtomSet<'alloc>>>,
    context: &mut T,
    from: BindingsIndex,
    to: BindingsIndex,
) -> Result<'alloc, ()>
where
    T: ParameterEarlyErrorsContext,
{
    for info in context_metadata.bindings_from_to(from, to) {
        context.declare(info.name, info.offset, &atoms.borrow())?;
    }

    Ok(())
}

/// Declare bindings to the body of lexical for-statement.
/// This method is an internal helper for EarlyErrorChecker
fn declare_lexical_for_body<'alloc>(
    context_metadata: &ContextMetadata,
    atoms: &Rc<RefCell<SourceAtomSet<'alloc>>>,
    context: &mut LexicalForBodyEarlyErrorsContext,
    index: BindingsIndex,
) -> Result<'alloc, ()> {
    for info in context_metadata.bindings_from(index) {
        match info.kind {
            BindingKind::Var => {
                context.declare_var(
                    info.name,
                    DeclarationKind::Var,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            _ => {
                panic!("Unexpected binding found {:?}", info);
            }
        }
    }

    Ok(())
}

/// Declare bindings to the head of lexical for-statement.
/// This method is an internal helper for EarlyErrorChecker
fn declare_lexical_for_head<'alloc>(
    context_metadata: &ContextMetadata,
    atoms: &Rc<RefCell<SourceAtomSet<'alloc>>>,
    context: &mut LexicalForHeadEarlyErrorsContext,
    from: BindingsIndex,
    to: BindingsIndex,
) -> Result<'alloc, ()> {
    for info in context_metadata.bindings_from_to(from, to) {
        match info.kind {
            BindingKind::Let => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Let,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            BindingKind::Const => {
                context.declare_lex(
                    info.name,
                    DeclarationKind::Const,
                    info.offset,
                    &atoms.borrow(),
                )?;
            }
            _ => {
                panic!("Unexpected binding found {:?}", info);
            }
        }
    }

    Ok(())
}

pub trait EarlyErrorChecker<'alloc> {
    fn context_metadata_mut(&mut self) -> &mut ContextMetadata;
    fn context_metadata(&self) -> &ContextMetadata;
    fn atoms(&self) -> &Rc<RefCell<SourceAtomSet<'alloc>>>;

    // Check Early Error for BindingIdentifier and note binding info to the
    // stack.
    fn on_binding_identifier(&mut self, token: &arena::Box<'alloc, Token>) -> Result<'alloc, ()> {
        let context = IdentifierEarlyErrorsContext::new();
        context.check_binding_identifier(token, &self.atoms().borrow())?;

        let name = token.value.as_atom();
        let offset = token.loc.start;

        if let Some(info) = self.context_metadata_mut().last_binding() {
            debug_assert!(info.offset < offset);
        }

        self.context_metadata_mut().push_binding(BindingInfo {
            name,
            offset,
            kind: BindingKind::Unknown,
        });

        Ok(())
    }

    // Check Early Error for IdentifierReference.
    fn on_identifier_reference(&self, token: &arena::Box<'alloc, Token>) -> Result<'alloc, ()> {
        let context = IdentifierEarlyErrorsContext::new();
        context.check_identifier_reference(token, &self.atoms().borrow())
    }

    // Check Early Error for LabelIdentifier and note binding info to the
    // stack
    fn on_label_identifier(&mut self, token: &arena::Box<'alloc, Token>) -> Result<'alloc, ()> {
        let context = IdentifierEarlyErrorsContext::new();

        let name = token.value.as_atom();
        let offset = token.loc.start;

        if let Some(info) = self.context_metadata_mut().last_binding() {
            debug_assert!(info.offset < offset);
        }

        // If the label is attached to a continue or break statement, its label info
        // is popped from the stack. See `continue_statement` and `break_statement` for more
        // information.
        self.context_metadata_mut().push_label(LabelInfo {
            name,
            offset,
            kind: LabelKind::Other,
        });

        context.check_label_identifier(token, &self.atoms().borrow())
    }

    /// Check Early Error for LabelledStatement.
    /// This should be called after handling the labelled body.
    fn check_labelled_statement(
        &mut self,
        name: SourceAtomSetIndex,
        start_of_label_offset: usize,
        start_of_statement_offset: usize,
    ) -> Result<'alloc, ()> {
        let label = self
            .context_metadata_mut()
            .find_label_at_offset(start_of_label_offset)
            .unwrap();

        let context = LabelledStatementEarlyErrorsContext::new(name, label.kind);
        let next_label_index = self
            .context_metadata_mut()
            .find_first_label(start_of_statement_offset);
        for info in self.context_metadata_mut().labels_from(next_label_index) {
            context.check_duplicate_label(info.name)?;
        }

        let break_or_continue_index = self
            .context_metadata_mut()
            .find_first_break_or_continue(start_of_label_offset);

        check_labelled_continue_to_non_loop(
            self.context_metadata_mut(),
            context,
            break_or_continue_index,
        )?;

        self.context_metadata_mut()
            .pop_labelled_breaks_and_continues_from_index(break_or_continue_index, name);
        Ok(())
    }

    // Static Semantics: Early Errors
    // https://tc39.es/ecma262/#sec-if-statement-static-semantics-early-errors
    // https://tc39.es/ecma262/#sec-semantics-static-semantics-early-errors
    // https://tc39.es/ecma262/#sec-with-statement-static-semantics-early-errors
    fn check_single_statement(&self, statement_start_offset: usize) -> Result<'alloc, ()> {
        // * It is a Syntax Error if IsLabelledFunction(Statement) is true.
        if is_labelled_function(self.context_metadata(), statement_start_offset) {
            return Err(ParseError::LabelledFunctionDeclInSingleStatement.into());
        }
        Ok(())
    }

    // Check bindings in Script. This is called at the end of a script,
    // after we have noted all bindings and identified that we are in a script.
    // Any remaining bindings should be legal in this context. Any labels within this
    // context are only valid here, and can be popped.
    fn check_script_bindings(&mut self) -> Result<'alloc, ()> {
        let mut context = ScriptEarlyErrorsContext::new();
        let index = BindingsIndex { index: 0 };
        declare_script_or_function(self.context_metadata(), self.atoms(), &mut context, index)?;
        self.context_metadata_mut().pop_bindings_from(index);

        let label_index = LabelIndex { index: 0 };
        self.context_metadata_mut().pop_labels_from(label_index);

        check_unhandled_break_or_continue(self.context_metadata_mut(), context, 0)?;

        Ok(())
    }

    // Check bindings in Module. This is called at the end of a module,
    // after we have noted all bindings and identified that we are in a Module.
    // Any remaining bindings should be legal in this context. Any labels within this
    // context are only valid here, and can be popped.
    fn check_module_bindings(&mut self) -> Result<'alloc, ()> {
        let mut context = ModuleEarlyErrorsContext::new();
        let index = BindingsIndex { index: 0 };
        declare_script_or_function(self.context_metadata(), self.atoms(), &mut context, index)?;
        self.context_metadata_mut().pop_bindings_from(index);

        let label_index = LabelIndex { index: 0 };
        self.context_metadata_mut().pop_labels_from(label_index);

        check_unhandled_break_or_continue(self.context_metadata_mut(), context, 0)?;

        Ok(())
    }

    // Check bindings in function with FormalParameters.
    fn check_function_bindings(
        &mut self,
        is_simple: bool,
        start_of_param_offset: usize,
        end_of_param_offset: usize,
    ) -> Result<'alloc, ()> {
        let mut param_context = if is_simple {
            FormalParametersEarlyErrorsContext::new_simple()
        } else {
            FormalParametersEarlyErrorsContext::new_non_simple()
        };

        let param_index = self
            .context_metadata_mut()
            .find_first_binding(start_of_param_offset);
        let body_index = self
            .context_metadata_mut()
            .find_first_binding(end_of_param_offset);
        declare_param(
            self.context_metadata(),
            self.atoms(),
            &mut param_context,
            param_index,
            body_index,
        )?;

        let mut body_context = FunctionBodyEarlyErrorsContext::new(param_context);
        declare_script_or_function(
            self.context_metadata(),
            self.atoms(),
            &mut body_context,
            body_index,
        )?;

        check_unhandled_break_or_continue(
            self.context_metadata_mut(),
            body_context,
            end_of_param_offset,
        )?;

        self.context_metadata_mut().pop_bindings_from(param_index);
        let label_index = self
            .context_metadata_mut()
            .find_first_label(start_of_param_offset);
        self.context_metadata_mut().pop_labels_from(label_index);

        Ok(())
    }

    // Check bindings in function with UniqueFormalParameters.
    fn check_unique_function_bindings(
        &mut self,
        start_of_param_offset: usize,
        end_of_param_offset: usize,
    ) -> Result<'alloc, ()> {
        let mut param_context = UniqueFormalParametersEarlyErrorsContext::new();

        let param_index = self
            .context_metadata_mut()
            .find_first_binding(start_of_param_offset);
        let body_index = self
            .context_metadata_mut()
            .find_first_binding(end_of_param_offset);
        declare_param(
            self.context_metadata(),
            self.atoms(),
            &mut param_context,
            param_index,
            body_index,
        )?;

        let mut body_context = UniqueFunctionBodyEarlyErrorsContext::new(param_context);
        declare_script_or_function(
            self.context_metadata(),
            self.atoms(),
            &mut body_context,
            body_index,
        )?;

        self.context_metadata_mut().pop_bindings_from(param_index);

        let label_index = self
            .context_metadata_mut()
            .find_first_label(start_of_param_offset);
        self.context_metadata_mut().pop_labels_from(label_index);

        check_unhandled_break_or_continue(
            self.context_metadata_mut(),
            body_context,
            end_of_param_offset,
        )?;

        Ok(())
    }

    // Check bindings in Block.
    fn check_block_bindings(&mut self, start_of_block_offset: usize) -> Result<'alloc, ()> {
        let mut context = BlockEarlyErrorsContext::new();
        let index = self
            .context_metadata_mut()
            .find_first_binding(start_of_block_offset);
        declare_block(self.context_metadata(), self.atoms(), &mut context, index)?;
        self.context_metadata_mut().pop_lexical_bindings_from(index);

        Ok(())
    }

    // Check bindings in CaseBlock of switch-statement.
    fn check_case_block_binding(&mut self, start_of_block_offset: usize) -> Result<'alloc, ()> {
        let mut context = CaseBlockEarlyErrorsContext::new();

        let index = self
            .context_metadata_mut()
            .find_first_binding(start_of_block_offset);
        // Check bindings in CaseBlock of switch-statement.
        declare_block(self.context_metadata(), self.atoms(), &mut context, index)?;
        self.context_metadata_mut().pop_lexical_bindings_from(index);

        self.context_metadata_mut()
            .pop_unlabelled_breaks_from(start_of_block_offset);

        Ok(())
    }

    // Check bindings in Catch and Block.
    fn check_catch_bindings(
        &mut self,
        is_simple: bool,
        start_of_bindings_offset: usize,
        end_of_bindings_offset: usize,
    ) -> Result<'alloc, ()> {
        let mut param_context = if is_simple {
            CatchParameterEarlyErrorsContext::new_with_binding_identifier()
        } else {
            CatchParameterEarlyErrorsContext::new_with_binding_pattern()
        };

        let param_index = self
            .context_metadata_mut()
            .find_first_binding(start_of_bindings_offset);
        let body_index = self
            .context_metadata_mut()
            .find_first_binding(end_of_bindings_offset);
        declare_param(
            self.context_metadata(),
            self.atoms(),
            &mut param_context,
            param_index,
            body_index,
        )?;

        let mut block_context = CatchBlockEarlyErrorsContext::new(param_context);
        declare_block(
            self.context_metadata(),
            self.atoms(),
            &mut block_context,
            body_index,
        )?;
        self.context_metadata_mut()
            .pop_lexical_bindings_from(param_index);

        Ok(())
    }

    // Check bindings in Catch with no parameter and Block.
    fn check_catch_no_param_bindings(
        &mut self,
        start_of_catch_offset: usize,
    ) -> Result<'alloc, ()> {
        let body_index = self
            .context_metadata_mut()
            .find_first_binding(start_of_catch_offset);

        let param_context = CatchParameterEarlyErrorsContext::new_with_binding_identifier();
        let mut block_context = CatchBlockEarlyErrorsContext::new(param_context);
        declare_block(
            self.context_metadata(),
            self.atoms(),
            &mut block_context,
            body_index,
        )?;
        self.context_metadata_mut()
            .pop_lexical_bindings_from(body_index);

        Ok(())
    }

    // Check bindings in lexical for-statement.
    fn check_lexical_for_bindings(
        &mut self,
        start_of_bindings_offset: usize,
        end_of_bindings_offset: usize,
    ) -> Result<'alloc, ()> {
        let mut head_context = LexicalForHeadEarlyErrorsContext::new();

        let head_index = self
            .context_metadata_mut()
            .find_first_binding(start_of_bindings_offset);
        let body_index = self
            .context_metadata_mut()
            .find_first_binding(end_of_bindings_offset);
        declare_lexical_for_head(
            self.context_metadata(),
            self.atoms(),
            &mut head_context,
            head_index,
            body_index,
        )?;

        let mut body_context = LexicalForBodyEarlyErrorsContext::new(head_context);
        declare_lexical_for_body(
            self.context_metadata(),
            self.atoms(),
            &mut body_context,
            body_index,
        )?;
        self.context_metadata_mut()
            .pop_lexical_bindings_from(head_index);

        Ok(())
    }
}

pub struct EarlyErrorBuilder<'alloc> {
    context_metadata: ContextMetadata,

    atoms: Rc<RefCell<SourceAtomSet<'alloc>>>,
}

impl<'alloc> EarlyErrorBuilder<'alloc> {
    pub fn new(atoms: Rc<RefCell<SourceAtomSet<'alloc>>>) -> Self {
        Self {
            context_metadata: ContextMetadata::new(),
            atoms,
        }
    }
}

impl<'alloc> EarlyErrorChecker<'alloc> for EarlyErrorBuilder<'alloc> {
    fn context_metadata_mut(&mut self) -> &mut ContextMetadata {
        &mut self.context_metadata
    }
    fn context_metadata(&self) -> &ContextMetadata {
        &self.context_metadata
    }
    fn atoms(&self) -> &Rc<RefCell<SourceAtomSet<'alloc>>> {
        &self.atoms
    }
}

impl<'alloc> EarlyErrorBuilder<'alloc> {

    fn collect_vec_from_results<T, C>(&self, results: C) -> Result<'alloc, ()>
    where
        C: IntoIterator<Item = Result<'alloc, T>>,
    {
        for result in results {
            result?;
        }
        Ok(())
    }

    // IdentifierReference : Identifier
    pub fn identifier_reference(
        &self,
        token: arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.on_identifier_reference(&token)?;
        Ok(())
    }

    // BindingIdentifier : Identifier
    pub fn binding_identifier(
        &mut self,
        token: arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.on_binding_identifier(&token)?;
        Ok(())
    }

    // BindingIdentifier : `yield`
    pub fn binding_identifier_yield(
        &mut self,
        token: arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.on_binding_identifier(&token)?;
        Ok(())
    }

    // BindingIdentifier : `await`
    pub fn binding_identifier_await(
        &mut self,
        token: arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.on_binding_identifier(&token)?;
        Ok(())
    }

    // LabelIdentifier : Identifier
    pub fn label_identifier(
        &mut self,
        token: arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.on_label_identifier(&token)?;
        Ok(())
    }

    // UpdateExpression : LeftHandSideExpression `++`
    pub fn post_increment_expr(
        &self,
        operand: &arena::Box<'alloc, Expression<'alloc>>,
        _operator_token: &arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.expression_to_simple_assignment_target2(&*operand)?;
        Ok(())
    }

    // UpdateExpression : LeftHandSideExpression `--`
    pub fn post_decrement_expr(
        &self,
        operand: &arena::Box<'alloc, Expression<'alloc>>,
        _operator_token: &arena::Box<'alloc, Token>,
    ) -> Result<'alloc, ()> {
        self.expression_to_simple_assignment_target2(&*operand)?;
        Ok(())
    }

    // UpdateExpression : `++` UnaryExpression
    pub fn pre_increment_expr(
        &self,
        _operator_token: &arena::Box<'alloc, Token>,
        operand: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        self.expression_to_simple_assignment_target2(&*operand)?;
        Ok(())
    }

    // UpdateExpression : `--` UnaryExpression
    pub fn pre_decrement_expr(
        &self,
        _operator_token: &arena::Box<'alloc, Token>,
        operand: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        self.expression_to_simple_assignment_target2(&*operand)?;
        Ok(())
    }

    fn expression_to_simple_assignment_target2(
        &self,
        expression: &Expression<'alloc>,
    ) -> Result<'alloc, ()> {
        match expression {
            // Static Semantics: AssignmentTargetType
            // https://tc39.es/ecma262/#sec-identifiers-static-semantics-assignmenttargettype
            Expression::IdentifierExpression(_) |
            Expression::MemberExpression(MemberExpression::StaticMemberExpression(_)) |
            Expression::MemberExpression(MemberExpression::ComputedMemberExpression(_)) => Ok(()),

            // Static Semantics: AssignmentTargetType
            // https://tc39.es/ecma262/#sec-static-semantics-static-semantics-assignmenttargettype
            //
            // CallExpression :
            //   CallExpression [ Expression ]
            //   CallExpression . IdentifierName
            //
            // 1. Return simple.
            Expression::CallExpression(CallExpression { .. }) => {
                return Err(ParseError::NotImplemented(
                    "Assignment to CallExpression is allowed for non-strict mode.",
                )
                .into());
            }

            _ => {
                return Err(ParseError::InvalidAssignmentTarget.into());
            }
        }
    }

    // AssignmentExpression : LeftHandSideExpression AssignmentOperator AssignmentExpression
    // AssignmentExpression : LeftHandSideExpression LogicalAssignmentOperator AssignmentExpression
    pub fn compound_assignment_expr(
        &self,
        left_hand_side: &arena::Box<'alloc, Expression<'alloc>>,
        _operator: &arena::Box<'alloc, CompoundAssignmentOperator>,
        _value: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        self.expression_to_simple_assignment_target2(&*left_hand_side)?;
        Ok(())
    }

    // ArrowParameters : CoverParenthesizedExpressionAndArrowParameterList
    pub fn uncover_arrow_parameters(
        &self,
        covered: &arena::Box<'alloc, CoverParenthesized<'alloc>>,
    ) -> Result<'alloc, ()> {
        match &**covered {
            CoverParenthesized::Expression { expression, .. } => Ok(self.expression_to_parameter_list(&expression)?),
            CoverParenthesized::Parameters(_) => Ok(()),
        }
    }

    // CoverParenthesizedExpressionAndArrowParameterList : `(` Expression `,` `)`
    // CoverParenthesizedExpressionAndArrowParameterList : `(` Expression `,` `...` BindingIdentifier `)`
    // CoverParenthesizedExpressionAndArrowParameterList : `(` Expression `,` `...` BindingPattern `)`
    pub fn expression_to_parameter_list2(
        &self,
        expression: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        // When the production
        // *ArrowParameters* `:` *CoverParenthesizedExpressionAndArrowParameterList*
        // is recognized the following grammar is used to refine the
        // interpretation of
        // *CoverParenthesizedExpressionAndArrowParameterList*:
        //
        //     ArrowFormalParameters[Yield, Await]:
        //         `(` UniqueFormalParameters[?Yield, ?Await] `)`
        match &**expression {
            Expression::BinaryExpression {
                operator: BinaryOperator::Comma { .. },
                left,
                right,
                ..
            } => {
                self.expression_to_parameter_list(left)?;
                Ok(self.expression_to_parameter(right)?)
            }
            other => Ok(self.unboxed_expression_to_parameter(other)?),
        }
    }

    pub fn expression_to_parameter_list(
        &self,
        expression: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        // When the production
        // *ArrowParameters* `:` *CoverParenthesizedExpressionAndArrowParameterList*
        // is recognized the following grammar is used to refine the
        // interpretation of
        // *CoverParenthesizedExpressionAndArrowParameterList*:
        //
        //     ArrowFormalParameters[Yield, Await]:
        //         `(` UniqueFormalParameters[?Yield, ?Await] `)`
        match &**expression {
            Expression::BinaryExpression {
                operator: BinaryOperator::Comma { .. },
                left,
                right,
                ..
            } => {
                self.expression_to_parameter_list(&left)?;
                Ok(self.expression_to_parameter(&right)?)
            }
            other => Ok(self.unboxed_expression_to_parameter(&other)?),
        }
    }

    fn object_property_to_binding_property(
        &self,
        op: &ObjectProperty<'alloc>,
    ) -> Result<'alloc, ()> {
        match op {
            ObjectProperty::NamedObjectProperty(NamedObjectProperty::DataProperty(
                DataProperty {
                    expression,
                    ..
                },
            )) => Ok(self.expression_to_parameter(&expression)?),

            ObjectProperty::NamedObjectProperty(NamedObjectProperty::MethodDefinition(_)) => {
                Err(ParseError::ObjectPatternWithMethod.into())
            }

            ObjectProperty::ShorthandProperty(ShorthandProperty {
                name: IdentifierExpression { .. },
                ..
            }) => {
                // TODO - CoverInitializedName can't be represented in an
                // ObjectProperty, but we need it here.
                Ok(())
            }

            ObjectProperty::SpreadProperty(_expression) => {
                Err(ParseError::ObjectPatternWithNonFinalRest.into())
            }
        }
    }

    /// Refine an instance of "*PropertyDefinition* : `...`
    /// *AssignmentExpression*" into a *BindingRestProperty*.
    fn spread_expression_to_rest_binding(
        &self,
        expression: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        Ok(match **expression {
            Expression::IdentifierExpression(IdentifierExpression { .. }) => (),
            _ => {
                return Err(ParseError::ObjectBindingPatternWithInvalidRest.into());
            }
        })
    }

    fn expression_to_binding_no_default(
        &self,
        expression: &Expression<'alloc>,
    ) -> Result<'alloc, ()> {
        match expression {
            Expression::IdentifierExpression(IdentifierExpression { .. }) => {
                Ok(())
            }

            Expression::ArrayExpression(ArrayExpression { elements, ..}) => {
                if let Some((rest, elems)) = elements.as_slice().split_last() {
                    self.collect_vec_from_results(elems.into_iter().map(|element| match element {
                        ArrayExpressionElement::Expression(expr) => {
                                Ok(self.expression_to_parameter(expr)?)
                            }
                        ArrayExpressionElement::SpreadElement(_expr) =>
                            // ([...a, b]) => {}
                            Err(ParseError::ArrayPatternWithNonFinalRest.into()),
                        ArrayExpressionElement::Elision { .. } => Ok(()),
                    }))?;
                    match rest {
                        ArrayExpressionElement::SpreadElement(rest) =>
                            self.expression_to_parameter_array(rest)?,
                        _ => ()
                    }
                }
                Ok(())
            }

            Expression::ObjectExpression(object) => Ok(self.object_expression_to_object_binding(object)?),

            _ => Err(ParseError::InvalidParameter.into()),
        }
    }

    fn expression_to_parameter_array(
        &self,
        expression: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        match &**expression {
            Expression::AssignmentExpression {
                binding,
                ..
            } => {
                self.assignment_target_to_binding(binding)?;
                let err: BoxedParseError =
                    ParseError::ArrayBindingPatternWithInvalidRest.into();
                Err(err)
            },

            other => Ok(self.expression_to_binding_no_default(other)?),
        }
    }

    /// Refine an *ObjectLiteral* into an *ObjectBindingPattern*.
    fn object_expression_to_object_binding(
        &self,
        object: &ObjectExpression<'alloc>,
    ) -> Result<'alloc, ()> {
        if let Some((rest, properties)) = object.properties.as_slice().split_last() {
            self.collect_vec_from_results(
                properties
                    .into_iter()
                    .map(|prop| self.object_property_to_binding_property(&**prop)),
            )?;
            if let ObjectProperty::SpreadProperty(rest) = &**rest {
                self.spread_expression_to_rest_binding(rest)?
            }
        }
        Ok(())
    }

    fn expression_to_parameter(
        &self,
        expression: &arena::Box<'alloc, Expression<'alloc>>,
    ) -> Result<'alloc, ()> {
        match &**expression {
            Expression::AssignmentExpression {
                binding,
                ..
            } => Ok(self.assignment_target_to_binding(binding)?),

            other => Ok(self.expression_to_binding_no_default(other)?),
        }
    }

    fn unboxed_expression_to_parameter(
        &self,
        expression: &Expression<'alloc>,
    ) -> Result<'alloc, ()> {
        match expression {
            Expression::AssignmentExpression {
                binding,
                ..
            } => Ok(self.assignment_target_to_binding(binding)?),

            other => Ok(self.expression_to_binding_no_default(other)?),
        }
    }

    /// Used when parsing `([a, b=2]=arr) =>` to reinterpret as parameter bindings
    /// the snippets `a` and `b=2`, which were previously parsed as assignment targets.
    fn assignment_target_maybe_default_to_binding(
        &self,
        target: &AssignmentTargetMaybeDefault<'alloc>,
    ) -> Result<'alloc, ()> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTarget(target) => Ok(self.assignment_target_to_binding(target)?),

            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
                AssignmentTargetWithDefault { binding, .. },
            ) => Ok(self.assignment_target_to_binding(binding)?),
        }
    }

    fn assignment_target_property_to_binding_property(
        &self,
        target: &AssignmentTargetProperty<'alloc>,
    ) -> Result<'alloc, ()> {
        Ok(match target {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                AssignmentTargetPropertyIdentifier {
                    binding: AssignmentTargetIdentifier { .. },
                    ..
                },
            ) => (),

            AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                AssignmentTargetPropertyProperty { binding, .. },
            ) => self.assignment_target_maybe_default_to_binding(binding)?,
        })
    }

    /// Refine an AssignmentRestProperty into a BindingRestProperty.
    fn assignment_rest_property_to_binding_identifier(
        &self,
        target: &AssignmentTarget<'alloc>,
    ) -> Result<'alloc, ()> {
        match target {
            // ({...x} = dv) => {}
            AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::AssignmentTargetIdentifier(AssignmentTargetIdentifier { .. }),
            ) => Ok(()),

            // ({...x.y} = dv) => {}
            _ => Err(ParseError::ObjectBindingPatternWithInvalidRest.into()),
        }
    }

    /// Refine the left-hand side of `=` to a parameter binding. The spec says:
    ///
    /// > When the production *ArrowParameters* :
    /// > *CoverParenthesizedExpressionAndArrowParameterList* is recognized,
    /// > the following grammar is used to refine the interpretation of
    /// > *CoverParenthesizedExpressionAndArrowParameterList*:
    /// >
    /// > *ArrowFormalParameters*\[Yield, Await\] :
    /// > `(` *UniqueFormalParameters*\[?Yield, ?Await\] `)`
    ///
    /// Of course, rather than actually reparsing the arrow function parameters,
    /// we work by refining the AST we already built.
    ///
    /// When parsing `(a = 1, [b, c] = obj) => {}`, the assignment targets `a`
    /// and `[b, c]` are passed to this method.
    fn assignment_target_to_binding(
        &self,
        target: &AssignmentTarget<'alloc>,
    ) -> Result<'alloc, ()> {
        match target {
            // (a = dv) => {}
            AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::AssignmentTargetIdentifier(AssignmentTargetIdentifier {
                    ..
                }),
            ) => Ok(()),

            // This case is always an early SyntaxError.
            // (a.x = dv) => {}
            // (a[i] = dv) => {}
            AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::MemberAssignmentTarget(_),
            ) => Err(ParseError::InvalidParameter.into()),

            // ([a, b] = dv) => {}
            AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ArrayAssignmentTarget(ArrayAssignmentTarget {
                    elements,
                    rest,
                    ..
                }),
            ) => {
                let elements: &arena::Vec<'alloc, Option<AssignmentTargetMaybeDefault<'alloc>>> =
                    &elements;
                self.collect_vec_from_results(elements.into_iter().map(|maybe_target| {
                    maybe_target.as_ref()
                        .map(|target| self.assignment_target_maybe_default_to_binding(target))
                        .transpose()
                }))?;
                if let Some(rest_target) = rest {
                    self.assignment_target_to_binding(rest_target)?
                };
                Ok(())
            }

            // ({a, b: c} = dv) => {}
            AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ObjectAssignmentTarget(ObjectAssignmentTarget {
                    properties,
                    rest,
                    ..
                }),
            ) => {
                self.collect_vec_from_results(properties.into_iter().map(|target| {
                    self.assignment_target_property_to_binding_property(target)
                }))?;

                if let Some(rest_target) = rest {
                    self.assignment_rest_property_to_binding_identifier(rest_target)?
                };
                Ok(())
            }
        }
    }

}
