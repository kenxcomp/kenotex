/// L4 Atom: Syntax highlighter wrapping syntect for fenced code block highlighting.

use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxTokenKind {
    Comment,
    Keyword,
    StringLiteral,
    TypeName,
    Function,
    Constant,
    Operator,
    Punctuation,
    Variable,
    Plain,
}

#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub text: String,
    pub kind: SyntaxTokenKind,
}

/// Bundles ParseState and ScopeStack so scope context carries across lines.
pub struct CodeParseState {
    pub parse_state: ParseState,
    pub scope_stack: ScopeStack,
}

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    // Pre-computed scopes for fast prefix matching
    scope_comment: Scope,
    scope_keyword: Scope,
    scope_keyword_operator: Scope,
    scope_storage_type: Scope,
    scope_string: Scope,
    scope_constant: Scope,
    scope_entity_name_function: Scope,
    scope_entity_name_type: Scope,
    scope_variable: Scope,
    scope_punctuation: Scope,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_nonewlines();

        Self {
            syntax_set,
            scope_comment: Scope::new("comment").unwrap(),
            scope_keyword: Scope::new("keyword").unwrap(),
            scope_keyword_operator: Scope::new("keyword.operator").unwrap(),
            scope_storage_type: Scope::new("storage.type").unwrap(),
            scope_string: Scope::new("string").unwrap(),
            scope_constant: Scope::new("constant").unwrap(),
            scope_entity_name_function: Scope::new("entity.name.function").unwrap(),
            scope_entity_name_type: Scope::new("entity.name.type").unwrap(),
            scope_variable: Scope::new("variable").unwrap(),
            scope_punctuation: Scope::new("punctuation").unwrap(),
        }
    }

    pub fn find_syntax(&self, lang: &str) -> Option<&SyntaxReference> {
        self.syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_name(lang))
    }

    pub fn create_parse_state(&self, syntax: &SyntaxReference) -> CodeParseState {
        CodeParseState {
            parse_state: ParseState::new(syntax),
            scope_stack: ScopeStack::new(),
        }
    }

    pub fn tokenize_line(
        &self,
        line: &str,
        code_state: &mut CodeParseState,
    ) -> Vec<SyntaxToken> {
        let ops = match code_state.parse_state.parse_line(line, &self.syntax_set) {
            Ok(ops) => ops,
            Err(_) => {
                return vec![SyntaxToken {
                    text: line.to_string(),
                    kind: SyntaxTokenKind::Plain,
                }];
            }
        };

        let mut tokens = Vec::new();
        let mut prev_byte = 0;

        for (byte_offset, op) in &ops {
            let byte_offset = *byte_offset;
            if byte_offset > prev_byte {
                let text = &line[prev_byte..byte_offset];
                if !text.is_empty() {
                    let kind = self.classify_scope(&code_state.scope_stack);
                    tokens.push(SyntaxToken {
                        text: text.to_string(),
                        kind,
                    });
                }
            }
            code_state.scope_stack.apply(op).ok();
            prev_byte = byte_offset;
        }

        // Remaining text after last operation
        if prev_byte < line.len() {
            let text = &line[prev_byte..];
            if !text.is_empty() {
                let kind = self.classify_scope(&code_state.scope_stack);
                tokens.push(SyntaxToken {
                    text: text.to_string(),
                    kind,
                });
            }
        }

        // Handle empty line
        if tokens.is_empty() && line.is_empty() {
            tokens.push(SyntaxToken {
                text: String::new(),
                kind: SyntaxTokenKind::Plain,
            });
        }

        tokens
    }

    fn classify_scope(&self, stack: &ScopeStack) -> SyntaxTokenKind {
        // Walk scopes from top (most specific) to bottom
        for scope in stack.as_slice().iter().rev() {
            // Check comment first (highest priority)
            if self.scope_comment.is_prefix_of(*scope) {
                return SyntaxTokenKind::Comment;
            }
            // keyword.operator -> Operator (before keyword check)
            if self.scope_keyword_operator.is_prefix_of(*scope) {
                return SyntaxTokenKind::Operator;
            }
            // keyword (non-operator)
            if self.scope_keyword.is_prefix_of(*scope) {
                return SyntaxTokenKind::Keyword;
            }
            // storage.type -> Keyword
            if self.scope_storage_type.is_prefix_of(*scope) {
                return SyntaxTokenKind::Keyword;
            }
            // string
            if self.scope_string.is_prefix_of(*scope) {
                return SyntaxTokenKind::StringLiteral;
            }
            // constant
            if self.scope_constant.is_prefix_of(*scope) {
                return SyntaxTokenKind::Constant;
            }
            // entity.name.function
            if self.scope_entity_name_function.is_prefix_of(*scope) {
                return SyntaxTokenKind::Function;
            }
            // entity.name.type
            if self.scope_entity_name_type.is_prefix_of(*scope) {
                return SyntaxTokenKind::TypeName;
            }
            // variable
            if self.scope_variable.is_prefix_of(*scope) {
                return SyntaxTokenKind::Variable;
            }
            // punctuation
            if self.scope_punctuation.is_prefix_of(*scope) {
                return SyntaxTokenKind::Punctuation;
            }
        }
        SyntaxTokenKind::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_highlighter() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.find_syntax("rs").is_some());
        assert!(highlighter.find_syntax("python").is_some());
    }

    #[test]
    fn test_find_syntax_known_languages() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.find_syntax("rust").is_some());
        assert!(highlighter.find_syntax("py").is_some());
        assert!(highlighter.find_syntax("js").is_some());
    }

    #[test]
    fn test_find_syntax_unknown_language() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.find_syntax("nonexistent_lang_xyz").is_none());
    }

    #[test]
    fn test_tokenize_rust_let() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        let tokens = highlighter.tokenize_line("let x = 42;", &mut state);
        assert!(!tokens.is_empty());

        // Should contain at least a keyword token for "let"
        assert!(tokens.iter().any(|t| t.kind == SyntaxTokenKind::Keyword));
        // Should contain a constant token for "42"
        assert!(tokens.iter().any(|t| t.kind == SyntaxTokenKind::Constant));
    }

    #[test]
    fn test_tokenize_comment() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        let tokens = highlighter.tokenize_line("// this is a comment", &mut state);
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.kind == SyntaxTokenKind::Comment));
    }

    #[test]
    fn test_tokenize_string() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        let tokens = highlighter.tokenize_line("let s = \"hello\";", &mut state);
        assert!(!tokens.is_empty());
        assert!(tokens
            .iter()
            .any(|t| t.kind == SyntaxTokenKind::StringLiteral));
    }

    #[test]
    fn test_tokenize_empty_line() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        let tokens = highlighter.tokenize_line("", &mut state);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, SyntaxTokenKind::Plain);
    }

    #[test]
    fn test_tokenize_preserves_text() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        let line = "fn main() { }";
        let tokens = highlighter.tokenize_line(line, &mut state);
        let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(reconstructed, line);
    }

    #[test]
    fn test_multiline_state_continuity() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.find_syntax("rs").unwrap();
        let mut state = highlighter.create_parse_state(syntax);

        // Parse multiple lines to test state is carried across calls.
        // After a `let` statement, the parser state should allow
        // subsequent lines to be parsed in context.
        let tokens1 = highlighter.tokenize_line("let x = 42;", &mut state);
        let tokens2 = highlighter.tokenize_line("let y = \"hello\";", &mut state);

        // First line should have keyword and constant
        assert!(tokens1.iter().any(|t| t.kind == SyntaxTokenKind::Keyword));
        assert!(tokens1.iter().any(|t| t.kind == SyntaxTokenKind::Constant));
        // Second line should have keyword and string
        assert!(tokens2.iter().any(|t| t.kind == SyntaxTokenKind::Keyword));
        assert!(tokens2
            .iter()
            .any(|t| t.kind == SyntaxTokenKind::StringLiteral));
    }

    #[test]
    fn test_find_syntax_shell() {
        let hl = SyntaxHighlighter::new();
        // "bash" or "shell" should resolve to a valid syntax
        assert!(
            hl.find_syntax("bash").is_some() || hl.find_syntax("shell").is_some(),
            "Neither 'bash' nor 'shell' found a syntax definition"
        );
    }

    #[test]
    fn test_find_syntax_case_variations() {
        let hl = SyntaxHighlighter::new();
        // syntect's find_syntax_by_token is case-sensitive, but find_syntax_by_name
        // may handle case differently. Test the typical lowercase tokens.
        assert!(hl.find_syntax("json").is_some());
        assert!(hl.find_syntax("yaml").is_some());
        assert!(hl.find_syntax("html").is_some());
        assert!(hl.find_syntax("css").is_some());
        assert!(hl.find_syntax("markdown").is_some() || hl.find_syntax("md").is_some());
    }

    #[test]
    fn test_tokenize_function_definition() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl.find_syntax("rs").unwrap();
        let mut state = hl.create_parse_state(syntax);

        let tokens = hl.tokenize_line("fn hello_world() {}", &mut state);
        assert!(!tokens.is_empty());
        // "fn" should be a keyword
        assert!(
            tokens.iter().any(|t| t.kind == SyntaxTokenKind::Keyword),
            "Expected Keyword token for 'fn'"
        );
    }

    #[test]
    fn test_tokenize_python() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl.find_syntax("py").unwrap();
        let mut state = hl.create_parse_state(syntax);

        let tokens = hl.tokenize_line("def foo(x):", &mut state);
        assert!(!tokens.is_empty());
        // "def" should be a keyword in Python
        assert!(tokens.iter().any(|t| t.kind == SyntaxTokenKind::Keyword));
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl.find_syntax("rs").unwrap();
        let mut state = hl.create_parse_state(syntax);

        let tokens = hl.tokenize_line("    ", &mut state);
        // Whitespace-only lines should produce tokens that reconstruct to the original
        let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(reconstructed, "    ");
    }

    #[test]
    fn test_tokenize_operator() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl.find_syntax("rs").unwrap();
        let mut state = hl.create_parse_state(syntax);

        let tokens = hl.tokenize_line("let x = a + b;", &mut state);
        // Should detect operators (keyword.operator scope)
        let has_operator = tokens.iter().any(|t| t.kind == SyntaxTokenKind::Operator);
        let has_punctuation = tokens
            .iter()
            .any(|t| t.kind == SyntaxTokenKind::Punctuation);
        // At least one of operator or punctuation should be present
        assert!(
            has_operator || has_punctuation,
            "Expected Operator or Punctuation tokens"
        );
    }

    #[test]
    fn test_syntax_token_kind_clone_eq() {
        let a = SyntaxTokenKind::Comment;
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(SyntaxTokenKind::Keyword, SyntaxTokenKind::Comment);
    }

    #[test]
    fn test_tokenize_multiple_lines_preserves_all_text() {
        let hl = SyntaxHighlighter::new();
        let syntax = hl.find_syntax("rs").unwrap();
        let mut state = hl.create_parse_state(syntax);

        let lines = vec![
            "use std::io;",
            "",
            "fn main() {",
            "    let x = 42;",
            "    println!(\"hello\");",
            "}",
        ];

        for line in &lines {
            let tokens = hl.tokenize_line(line, &mut state);
            let reconstructed: String = tokens.iter().map(|t| t.text.as_str()).collect();
            assert_eq!(
                &reconstructed, line,
                "Text reconstruction mismatch for line: {:?}",
                line
            );
        }
    }
}
