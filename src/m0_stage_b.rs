//! Deterministic structural classification, formatting, and editor validation
//! for the M0 distinction constitution.
//!
//! Stage B consumes only the lossless Stage-A tree. It classifies source form;
//! it does not resolve terms, assign referent identities, create assertion
//! occurrences, or elaborate relational content.

use crate::m0_stage_a::{
    DiagnosticCode as StageADiagnosticCode, Document, IndentationGroup, LayoutLine, Line,
    Punctuation, SourceSpan, Token, TokenKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockClass {
    Enumeration,
    FocusedProjection,
    AssertionOccurrence,
    RelationContract,
    UniversalLaw,
    DerivationRule,
    Invariant,
    Query,
    Goal,
    Observation,
    Requirement,
    Assumption,
    Intention,
    Effect,
    Procedure,
    Transition,
    Delta,
    StructuralEscape,
    UnresolvedStructuralForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildForm {
    BareTerm,
    Binding,
    Membership,
    RelationalContent,
    AssertionOccurrence,
    RelationContract,
    UniversalLaw,
    DerivationRule,
    Invariant,
    Query,
    Goal,
    Observation,
    Requirement,
    Assumption,
    Intention,
    Effect,
    Procedure,
    Transition,
    Delta,
    StructuralEscape,
    UnresolvedStructuralForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    StageATabIndentation,
    StageANoncanonicalIndentation,
    PersistedDoubleColon,
    MembershipInAlias,
    MembershipMemberOfAlias,
    EmptyBlock,
    MultipleChildIndentations,
    UnclassifiableChild,
    NotRoleLabelledStructuralEscape,
    WouldReclassifyEnumeration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: SourceSpan,
    pub repair: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedBlock {
    pub line: usize,
    pub header: SourceSpan,
    pub class: BlockClass,
    pub child_forms: Vec<ChildForm>,
    pub children: Vec<ClassifiedBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    pub blocks: Vec<ClassifiedBlock>,
    pub statements: Vec<ClassifiedStatement>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementClass {
    GroundingTerm,
    Binding,
    Membership,
    RelationalContent,
    AssertionOccurrence,
    RelationContract,
    UniversalLaw,
    DerivationRule,
    Invariant,
    Query,
    Goal,
    Observation,
    Requirement,
    Assumption,
    Intention,
    Effect,
    Procedure,
    Transition,
    Delta,
    StructuralEscape,
    UnresolvedStructuralForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClassifiedStatement {
    pub line: usize,
    pub span: SourceSpan,
    pub class: StatementClass,
}

impl Classification {
    pub fn is_accepted(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Classifies every indentation group, or returns a finite exact diagnostic.
pub fn classify(document: &Document) -> Classification {
    let mut diagnostics = stage_a_diagnostics(document);
    reject_membership_aliases(document, &mut diagnostics);

    let mut blocks = Vec::new();
    let mut statements = Vec::new();
    if diagnostics.is_empty() {
        classify_group(
            document,
            &document.root,
            &mut blocks,
            &mut statements,
            &mut diagnostics,
        );
    }

    Classification {
        blocks,
        statements,
        diagnostics,
    }
}

fn stage_a_diagnostics(document: &Document) -> Vec<Diagnostic> {
    document
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let (code, repair) = match diagnostic.code {
                StageADiagnosticCode::TabIndentation => (
                    DiagnosticCode::StageATabIndentation,
                    "replace each tab with the canonical number of spaces",
                ),
                StageADiagnosticCode::NoncanonicalIndentation => (
                    DiagnosticCode::StageANoncanonicalIndentation,
                    "indent each child exactly two spaces beyond its parent",
                ),
                StageADiagnosticCode::PersistedDoubleColon => (
                    DiagnosticCode::PersistedDoubleColon,
                    "replace `::` with `∈` before persistence or parsing",
                ),
            };
            Diagnostic {
                code,
                span: diagnostic.span,
                repair,
            }
        })
        .collect()
}

fn reject_membership_aliases(document: &Document, diagnostics: &mut Vec<Diagnostic>) {
    for line in &document.lines {
        let significant = line.tokens.as_slice();
        if is_in_membership_shape(document, significant) {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::MembershipInAlias,
                span: content_without_indent(line),
                repair: "write membership with `∈`",
            });
        } else if is_member_of_membership_shape(document, significant) {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::MembershipMemberOfAlias,
                span: content_without_indent(line),
                repair: "write membership with `∈`",
            });
        }
    }
}

fn is_in_membership_shape(document: &Document, tokens: &[Token]) -> bool {
    !trailing_colon(tokens)
        && tokens.iter().enumerate().any(|(index, token)| {
            index > 0
                && index + 1 < tokens.len()
                && token.kind == TokenKind::Name
                && token_text(document, *token) == "in"
        })
}

fn is_member_of_membership_shape(document: &Document, tokens: &[Token]) -> bool {
    !trailing_colon(tokens)
        && tokens.windows(2).enumerate().any(|(index, pair)| {
            index > 0
                && index + 2 < tokens.len()
                && pair[0].kind == TokenKind::Name
                && pair[1].kind == TokenKind::Name
                && token_text(document, pair[0]) == "member"
                && token_text(document, pair[1]) == "of"
        })
}

fn trailing_colon(tokens: &[Token]) -> bool {
    tokens
        .last()
        .is_some_and(|token| token.kind == TokenKind::Punctuation(Punctuation::Colon))
}

fn classify_group(
    document: &Document,
    group: &IndentationGroup,
    blocks: &mut Vec<ClassifiedBlock>,
    statements: &mut Vec<ClassifiedStatement>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for layout_line in &group.lines {
        if layout_line.children.is_empty() {
            let line = &document.lines[layout_line.line];
            if line.tokens.is_empty() {
                continue;
            }
            let Some(form) = line_form(document, line, false) else {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::UnclassifiableChild,
                    span: content_without_indent(line),
                    repair: "use an explicit role-labelled structural form",
                });
                return;
            };
            statements.push(ClassifiedStatement {
                line: layout_line.line,
                span: content_without_indent(line),
                class: statement_class(form),
            });
            continue;
        }
        match classify_block(document, layout_line, diagnostics) {
            Some(block) => blocks.push(block),
            None => return,
        }
    }
}

fn classify_block(
    document: &Document,
    layout_line: &LayoutLine,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ClassifiedBlock> {
    let header_line = &document.lines[layout_line.line];
    let header = content_without_indent(header_line);
    if layout_line.children.len() != 1 {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::MultipleChildIndentations,
            span: header,
            repair: "use one child indentation exactly two spaces beyond the parent",
        });
        return None;
    }

    let children = &layout_line.children[0];
    if children.lines.is_empty() {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::EmptyBlock,
            span: header,
            repair: "remove the empty block or add one structural child",
        });
        return None;
    }

    let mut child_forms = Vec::with_capacity(children.lines.len());
    let mut nested_blocks = Vec::new();
    for child in &children.lines {
        let child_line = &document.lines[child.line];
        let Some(form) = line_form(document, child_line, !child.children.is_empty()) else {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::UnclassifiableChild,
                span: content_without_indent(child_line),
                repair: "use an explicit role-labelled structural form",
            });
            return None;
        };
        child_forms.push(form);
        if !child.children.is_empty() {
            let block = classify_block(document, child, diagnostics)?;
            nested_blocks.push(block);
        }
    }

    let class = block_class(document, header_line, &child_forms);
    if class == BlockClass::FocusedProjection {
        for form in &mut child_forms {
            if *form == ChildForm::BareTerm {
                *form = ChildForm::Membership;
            }
        }
    }
    Some(ClassifiedBlock {
        line: layout_line.line,
        header,
        class,
        child_forms,
        children: nested_blocks,
    })
}

fn statement_class(form: ChildForm) -> StatementClass {
    match form {
        ChildForm::BareTerm => StatementClass::GroundingTerm,
        ChildForm::Binding => StatementClass::Binding,
        ChildForm::Membership => StatementClass::Membership,
        ChildForm::RelationalContent => StatementClass::RelationalContent,
        ChildForm::AssertionOccurrence => StatementClass::AssertionOccurrence,
        ChildForm::RelationContract => StatementClass::RelationContract,
        ChildForm::UniversalLaw => StatementClass::UniversalLaw,
        ChildForm::DerivationRule => StatementClass::DerivationRule,
        ChildForm::Invariant => StatementClass::Invariant,
        ChildForm::Query => StatementClass::Query,
        ChildForm::Goal => StatementClass::Goal,
        ChildForm::Observation => StatementClass::Observation,
        ChildForm::Requirement => StatementClass::Requirement,
        ChildForm::Assumption => StatementClass::Assumption,
        ChildForm::Intention => StatementClass::Intention,
        ChildForm::Effect => StatementClass::Effect,
        ChildForm::Procedure => StatementClass::Procedure,
        ChildForm::Transition => StatementClass::Transition,
        ChildForm::Delta => StatementClass::Delta,
        ChildForm::StructuralEscape => StatementClass::StructuralEscape,
        ChildForm::UnresolvedStructuralForm => StatementClass::UnresolvedStructuralForm,
    }
}

fn block_class(document: &Document, header: &Line, children: &[ChildForm]) -> BlockClass {
    let first = first_name(document, header);
    if is_assertion_header(document, header) {
        return BlockClass::AssertionOccurrence;
    }
    if is_relation_contract_header(document, header) {
        return BlockClass::RelationContract;
    }
    if first == Some("law") {
        return BlockClass::UniversalLaw;
    }
    if line_ends_with_name(document, header, "if") {
        return BlockClass::DerivationRule;
    }
    if first == Some("invariant") {
        return BlockClass::Invariant;
    }
    if first.is_some_and(is_query_word) {
        return BlockClass::Query;
    }
    if first == Some("goal") {
        return BlockClass::Goal;
    }
    if first == Some("observe") {
        return BlockClass::Observation;
    }
    if first.is_some_and(is_requirement_word) {
        return BlockClass::Requirement;
    }
    if first.is_some_and(is_assumption_word) {
        return BlockClass::Assumption;
    }
    if first == Some("intend") {
        return BlockClass::Intention;
    }
    if is_effect_form(document, header) {
        return BlockClass::Effect;
    }
    if first.is_some_and(is_procedure_word) {
        return BlockClass::Procedure;
    }
    if first == Some("on")
        || has_token_pair(header, Punctuation::Tilde, Punctuation::GreaterThan)
        || children.contains(&ChildForm::Transition)
    {
        return BlockClass::Transition;
    }
    if is_revision_header(document, header) && children.iter().all(|form| *form == ChildForm::Delta)
    {
        return BlockClass::Delta;
    }
    if is_structural_escape_header(document, header) {
        return BlockClass::StructuralEscape;
    }
    if children.iter().all(|form| *form == ChildForm::BareTerm) {
        return BlockClass::Enumeration;
    }
    if children.iter().any(|form| {
        matches!(
            form,
            ChildForm::BareTerm
                | ChildForm::Binding
                | ChildForm::Membership
                | ChildForm::RelationalContent
                | ChildForm::AssertionOccurrence
                | ChildForm::RelationContract
                | ChildForm::Transition
                | ChildForm::Effect
                | ChildForm::UnresolvedStructuralForm
        )
    }) {
        return BlockClass::FocusedProjection;
    }
    BlockClass::UnresolvedStructuralForm
}

fn line_form(document: &Document, line: &Line, has_children: bool) -> Option<ChildForm> {
    if line.tokens.is_empty() {
        return None;
    }
    let first = first_name(document, line);
    if first_name(document, line) == Some("assert")
        || has_children && line_ends_with_name(document, line, "asserts")
    {
        return Some(ChildForm::AssertionOccurrence);
    }
    if is_relation_contract_header(document, line) {
        return Some(ChildForm::RelationContract);
    }
    if first == Some("law") {
        return Some(ChildForm::UniversalLaw);
    }
    if line_ends_with_name(document, line, "if") {
        return Some(ChildForm::DerivationRule);
    }
    if first == Some("invariant") {
        return Some(ChildForm::Invariant);
    }
    if first.is_some_and(is_query_word) {
        return Some(ChildForm::Query);
    }
    if first == Some("goal") {
        return Some(ChildForm::Goal);
    }
    if first == Some("observe") {
        return Some(ChildForm::Observation);
    }
    if first.is_some_and(is_requirement_word) {
        return Some(ChildForm::Requirement);
    }
    if first.is_some_and(is_assumption_word) {
        return Some(ChildForm::Assumption);
    }
    if first == Some("intend") {
        return Some(ChildForm::Intention);
    }
    if is_effect_form(document, line) {
        return Some(ChildForm::Effect);
    }
    if first.is_some_and(is_procedure_word) {
        return Some(ChildForm::Procedure);
    }
    if first == Some("on") || has_token_pair(line, Punctuation::Tilde, Punctuation::GreaterThan) {
        return Some(ChildForm::Transition);
    }
    if is_structural_escape_header(document, line) {
        return Some(ChildForm::StructuralEscape);
    }
    if first_token_is(line, Punctuation::Plus) || first_token_is(line, Punctuation::Minus) {
        return Some(ChildForm::Delta);
    }
    if line_has_top_level_symbol(document, line, "∈") {
        return Some(ChildForm::Membership);
    }
    if has_top_level_binding_colon(line) {
        return Some(ChildForm::Binding);
    }
    if has_token_pair(line, Punctuation::Minus, Punctuation::GreaterThan) {
        return Some(ChildForm::RelationContract);
    }
    if line
        .tokens
        .iter()
        .enumerate()
        .any(|(index, token)| is_canonical_infix_operator(document, line, index, *token))
    {
        return Some(ChildForm::RelationalContent);
    }
    if line.tokens.len() == 1 && line.tokens[0].kind == TokenKind::Name {
        return Some(ChildForm::BareTerm);
    }
    if line
        .tokens
        .iter()
        .any(|token| token.kind == TokenKind::Punctuation(Punctuation::Question))
    {
        return Some(ChildForm::RelationalContent);
    }
    Some(ChildForm::UnresolvedStructuralForm)
}

fn is_canonical_infix_operator(
    document: &Document,
    line: &Line,
    index: usize,
    token: Token,
) -> bool {
    index > 0
        && index + 1 < line.tokens.len()
        && match token.kind {
            TokenKind::Punctuation(Punctuation::Slash) => {
                let before = &document.source[line.tokens[index - 1].span.end..token.span.start];
                let after = &document.source[token.span.end..line.tokens[index + 1].span.start];
                before.contains(' ') && after.contains(' ')
            }
            TokenKind::Punctuation(
                Punctuation::Equals
                | Punctuation::GreaterThan
                | Punctuation::LessThan
                | Punctuation::Plus
                | Punctuation::Minus
                | Punctuation::Star,
            ) => !adjacent_to_noncanonical_punctuation(line, index),
            _ => false,
        }
}

fn adjacent_to_noncanonical_punctuation(line: &Line, index: usize) -> bool {
    let token = line.tokens[index];
    line.tokens
        .get(index.wrapping_sub(1))
        .is_some_and(|previous| {
            previous.span.end == token.span.start
                && matches!(
                    previous.kind,
                    TokenKind::Punctuation(Punctuation::Colon | Punctuation::Equals)
                )
        })
        || line.tokens.get(index + 1).is_some_and(|next| {
            token.span.end == next.span.start
                && matches!(
                    next.kind,
                    TokenKind::Punctuation(
                        Punctuation::Colon
                            | Punctuation::GreaterThan
                            | Punctuation::LessThan
                            | Punctuation::Plus
                            | Punctuation::Minus
                            | Punctuation::Star
                            | Punctuation::Slash
                    )
                )
        })
}

fn first_name<'a>(document: &'a Document, line: &Line) -> Option<&'a str> {
    line.tokens
        .first()
        .filter(|token| token.kind == TokenKind::Name)
        .map(|token| token_text(document, *token))
}

fn is_query_word(word: &str) -> bool {
    matches!(
        word,
        "select" | "any" | "why" | "prevent" | "achieve" | "diff"
    )
}

fn is_assertion_header(document: &Document, line: &Line) -> bool {
    first_name(document, line) == Some("assert") || line_ends_with_name(document, line, "asserts")
}

fn is_requirement_word(word: &str) -> bool {
    matches!(word, "require" | "requires")
}

fn is_assumption_word(word: &str) -> bool {
    matches!(word, "assume" | "hypothesis")
}

fn is_procedure_word(word: &str) -> bool {
    matches!(word, "do" | "procedure")
}

fn is_effect_form(document: &Document, line: &Line) -> bool {
    first_name(document, line) == Some("effect")
        || line.tokens.iter().enumerate().any(|(index, token)| {
            token.kind == TokenKind::Punctuation(Punctuation::Exclamation)
                && !line.tokens.get(index + 1).is_some_and(|next| {
                    next.kind == TokenKind::Punctuation(Punctuation::Equals)
                        && token.span.end == next.span.start
                })
        })
}

fn is_relation_contract_header(document: &Document, line: &Line) -> bool {
    let mut names = line
        .tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Name)
        .map(|token| token_text(document, *token));
    names.next() == Some("relation") && names.next() == Some("contract")
}

fn is_revision_header(document: &Document, line: &Line) -> bool {
    line.tokens
        .iter()
        .any(|token| token.kind == TokenKind::Name && token_text(document, *token) == "from")
}

fn is_structural_escape_header(document: &Document, line: &Line) -> bool {
    first_name(document, line) == Some("form")
}

fn line_ends_with_name(document: &Document, line: &Line, name: &str) -> bool {
    line.tokens
        .last()
        .is_some_and(|token| token.kind == TokenKind::Name && token_text(document, *token) == name)
}

fn first_token_is(line: &Line, punctuation: Punctuation) -> bool {
    line.tokens
        .first()
        .is_some_and(|token| token.kind == TokenKind::Punctuation(punctuation))
}

fn has_token_pair(line: &Line, first: Punctuation, second: Punctuation) -> bool {
    line.tokens.windows(2).any(|tokens| {
        tokens[0].kind == TokenKind::Punctuation(first)
            && tokens[1].kind == TokenKind::Punctuation(second)
            && tokens[0].span.end == tokens[1].span.start
    })
}

fn has_top_level_binding_colon(line: &Line) -> bool {
    let mut depth = 0usize;
    let mut binding = None;
    for (index, token) in line.tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenParen)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenBracket)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenBrace) => depth += 1,
            TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseParen)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseBracket)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseBrace) => {
                depth = depth.saturating_sub(1);
            }
            TokenKind::Punctuation(Punctuation::Colon) if depth == 0 => {
                let next = line.tokens.get(index + 1);
                let adjacent_equals = next.is_some_and(|next| {
                    next.kind == TokenKind::Punctuation(Punctuation::Equals)
                        && token.span.end == next.span.start
                });
                let adjacent_colon = next.is_some_and(|next| {
                    next.kind == TokenKind::Punctuation(Punctuation::Colon)
                        && token.span.end == next.span.start
                });
                if index > 0 && index + 1 < line.tokens.len() && !adjacent_equals && !adjacent_colon
                {
                    if binding.is_some() {
                        return false;
                    }
                    binding = Some(index);
                }
            }
            _ => {}
        }
    }
    binding.is_some()
}

fn line_has_top_level_symbol(document: &Document, line: &Line, symbol: &str) -> bool {
    let mut depth = 0usize;
    for (index, token) in line.tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenParen)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenBracket)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::OpenBrace) => depth += 1,
            TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseParen)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseBracket)
            | TokenKind::Delimiter(crate::m0_stage_a::Delimiter::CloseBrace) => {
                depth = depth.saturating_sub(1);
            }
            TokenKind::Symbol
                if depth == 0
                    && index > 0
                    && index + 1 < line.tokens.len()
                    && token_text(document, *token) == symbol =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn token_text(document: &Document, token: Token) -> &str {
    &document.source[token.span.start..token.span.end]
}

fn content_without_indent(line: &Line) -> SourceSpan {
    SourceSpan {
        start: line.indentation.end,
        end: line.content_span.end,
    }
}

/// Canonicalizes accepted layout to two spaces and canonical source tokens.
/// This is structural only: line content is otherwise byte-for-byte retained.
pub fn format(document: &Document, classification: &Classification) -> Result<String, Diagnostic> {
    if let Some(diagnostic) = classification.diagnostics.first() {
        return Err(*diagnostic);
    }
    let mut output = String::new();
    format_group(document, &document.root, 0, classification, &mut output);
    Ok(output)
}

fn format_group(
    document: &Document,
    group: &IndentationGroup,
    depth: usize,
    classification: &Classification,
    output: &mut String,
) {
    let mut previous_block: Option<&ClassifiedBlock> = None;
    for line in &group.lines {
        let current_block = classification
            .blocks
            .iter()
            .find(|block| block.line == line.line);
        if let (Some(previous), Some(current)) = (previous_block, current_block)
            && separates_enumeration_and_focus(document, previous, current)
            && !output.ends_with("\n\n")
        {
            output.push('\n');
        }
        output.push_str(&"  ".repeat(depth));
        let source_line = &document.lines[line.line];
        output
            .push_str(&document.source[source_line.indentation.end..source_line.content_span.end]);
        output.push('\n');
        for children in &line.children {
            format_group(document, children, depth + 1, classification, output);
        }
        if current_block.is_some() {
            previous_block = current_block;
        }
    }
}

fn separates_enumeration_and_focus(
    document: &Document,
    previous: &ClassifiedBlock,
    current: &ClassifiedBlock,
) -> bool {
    let is_pair = matches!(
        (previous.class, current.class),
        (BlockClass::Enumeration, BlockClass::FocusedProjection)
            | (BlockClass::FocusedProjection, BlockClass::Enumeration)
    );
    is_pair
        && document.source[previous.header.start..previous.header.end]
            == document.source[current.header.start..current.header.end]
}

/// Canonically formats an already explicit role-labelled structural escape.
/// Stage B refuses to invent semantic role labels for shorthand; that belongs
/// to exact phrase resolution in Stage C.
pub fn format_role_labelled_escape(
    document: &Document,
    block: &ClassifiedBlock,
) -> Result<String, Diagnostic> {
    if block.class != BlockClass::StructuralEscape {
        return Err(Diagnostic {
            code: DiagnosticCode::NotRoleLabelledStructuralEscape,
            span: block.header,
            repair: "resolve the phrase, then provide an explicit role-labelled structural form",
        });
    }
    let layout_line = find_layout_line(&document.root, block.line)
        .expect("classified block retains its Stage-A layout line");
    let mut output = String::new();
    output.push_str(&document.source[block.header.start..block.header.end]);
    output.push('\n');
    for child in &layout_line.children[0].lines {
        let line = &document.lines[child.line];
        output.push_str("  ");
        output.push_str(&document.source[line.indentation.end..line.content_span.end]);
        output.push('\n');
    }
    Ok(output)
}

fn find_layout_line(group: &IndentationGroup, line_index: usize) -> Option<&LayoutLine> {
    for line in &group.lines {
        if line.line == line_index {
            return Some(line);
        }
        for children in &line.children {
            if let Some(found) = find_layout_line(children, line_index) {
                return Some(found);
            }
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorRewrite {
    pub source: String,
    pub replaced: Vec<SourceSpan>,
}

/// Applies the editor-only `::` completion before source is persisted or read.
/// Existing `∈`, binding colons, and quoted literals are left unchanged.
pub fn rewrite_editor_input(source: &str) -> EditorRewrite {
    let mut output = String::with_capacity(source.len());
    let mut replaced = Vec::new();
    let mut cursor = 0;
    let mut quote = None;
    let mut escaped = false;
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor remains in editor input");
        if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if quote == Some(character) && !escaped {
            quote = None;
        }
        if quote.is_none() && source[cursor..].starts_with("::") {
            output.push('∈');
            replaced.push(SourceSpan {
                start: cursor,
                end: cursor + 2,
            });
            cursor += 2;
            escaped = false;
            continue;
        }
        output.push(character);
        cursor += character.len_utf8();
        escaped = quote.is_some() && !escaped && character == '\\';
        if character != '\\' {
            escaped = false;
        }
    }
    EditorRewrite {
        source: output,
        replaced,
    }
}

/// Warns before an editor adds a non-bare child to an existing all-bare block.
pub fn warn_before_edit(
    _document: &Document,
    block: &ClassifiedBlock,
    prospective_child: &str,
) -> Option<Diagnostic> {
    if block.class != BlockClass::Enumeration {
        return None;
    }
    let prospective = crate::m0_stage_a::read(prospective_child);
    let line = prospective.lines.first()?;
    if line_form(&prospective, line, false) == Some(ChildForm::BareTerm) {
        return None;
    }
    Some(Diagnostic {
        code: DiagnosticCode::WouldReclassifyEnumeration,
        span: block.header,
        repair: "split enumeration members and focused relational content into separate blocks",
    })
}
