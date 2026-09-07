;; Rust Tree-sitter Tags Query

;; Structs
(struct_item
  name: (type_identifier) @definition.struct)

;; Enums
(enum_item
  name: (type_identifier) @definition.enum)

;; Traits
(trait_item
  name: (type_identifier) @definition.interface)

;; Implementations
(impl_item
  trait: (type_identifier)? @inherits
  type: (type_identifier) @scope.impl)

;; Functions
(function_item
  name: (identifier) @definition.function
  parameters: (parameters) @signature.params
  return_type: (type)? @signature.return)

;; Macro Definitions
(macro_definition
  name: (identifier) @definition.macro)

;; Calls
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression field: (field_identifier) @reference.call)
    (scoped_identifier name: (identifier) @reference.call)
  ])
