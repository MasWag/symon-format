; Rules for formatting SyMon.
;
; Formatting is specified here in terms of tree-sitter nodes. We select nodes
; with tree-sitter queries[^1] and then attach topiary formatting rules[^2] in
; the captures.
;
; See the Development section in README.md for a workflow on how to modify or
; extend these rules.

; [^1]: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
; [^2]: https://github.com/tweag/topiary#design

[
 (identifier)
 (number)
 (string)
] @leaf

(
  "var" @append_space
  "{" @append_indent_start @append_hardline
  (_)
  "}" @prepend_indent_end @prepend_hardline @append_hardline
)

(
  "init" @append_space
  "{" @append_indent_start @append_hardline
  (_)
  "}" @prepend_indent_end @prepend_hardline @append_hardline
)

(
  "signature" @append_space
  (identifier) @append_space
  "{" @append_indent_start @append_hardline
  "}" @prepend_indent_end @prepend_hardline @append_hardline
)

(string_definition
 [
  (identifier)
  ":" @append_space
  "string"
  ";" @append_hardline
  ]
)

(number_definition
 [
  (identifier)
  ":" @append_space
  "number"
  ";" @append_hardline
  ]
)

(def_expr
 [
  "expr" @append_space
  (identifier) @append_space
  "{" @append_hardline @append_indent_start
  (expr)
  "}" @prepend_indent_end @append_hardline @prepend_hardline
  ]
)

(atomic
 [
  (identifier)
  "(" @append_space
  ")" @prepend_space
  ]
)

(arg_list
 [
  (identifier)
  "," @append_space
  (identifier)
  ]
)

(guard_block
 "|" @append_space @prepend_space
 (_)
 ("|" @append_space @prepend_space
 (_))?
)

(constraint_list
 (constraint)
 "&&" @append_space @prepend_space
 (constraint_list)
)

(string_constraint
  (_)
  [
   "=="
   "!="
   ] @append_space @prepend_space
  (_)
)

(numeric_constraint
  (_)
  [
   "<="
   "<"
   "="
   "<>"
   ">"
   ">="
   ] @append_space @prepend_space
  (_)
)

(assignment
 (identifier)
 ":=" @append_space @prepend_space
 (numeric_expr)
)

(within
 "within" @append_space
 (timing_constraint) @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
)

(intervals
 [
  "("
  "["
 ]
 (numeric_expr)
 ","
 (numeric_expr)
 [
  ")"
  "]"
 ]
)

(half_guard
 "(" @append_space
 (numeric_expr) @prepend_space
 ")" @prepend_space
)

(conjunction
 (expr)
 "&&" @append_space @prepend_space
 (expr)
)

(disjunction
 (expr)
 "||" @append_space @prepend_space
 (expr)
)

(concat
 (expr)
 ";" @append_hardline
 (expr)
)

(optional_block
 "optional" @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
 )

(zero_or_more
 "zero_or_more" @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
)

(one_or_more
 "one_or_more" @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
)

(one_of
 "one_of" @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
 ("or" @prepend_space @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end)+
 )

(all_of
 "all_of" @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end
 ("and" @prepend_space @append_space
 "{" @append_hardline @append_indent_start
 (expr)
 "}" @prepend_hardline @prepend_indent_end)+
)

(paren_expr
 "("
 (expr) @prepend_space @append_space
 ")"
)

(numeric_expr
 (numeric_expr)
 [
  "+"
  "-"
 ] @prepend_space @append_space
 (numeric_expr)
)

[
  (line_comment)
  (variables)
  (initial_constraints)
  (signature)
  (def_expr)
  (expr)
] @allow_blank_line_before

(line_comment) @prepend_input_softline @append_input_softline @append_hardline
