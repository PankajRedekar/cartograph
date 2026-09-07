;; TypeScript & React Tree-sitter Tags Query

;; Interfaces
(interface_declaration
  name: (type_identifier) @definition.interface)

;; Type Aliases
(type_alias_declaration
  name: (type_identifier) @definition.type)

;; Classes
(class_declaration
  name: (type_identifier) @definition.class
  (class_heritage)? @inherits)

;; Methods
(method_definition
  name: (property_identifier) @definition.method
  parameters: (formal_parameters) @signature.params
  return_type: (type_annotation)? @signature.return)

;; Functions
(function_declaration
  name: (identifier) @definition.function
  parameters: (formal_parameters) @signature.params
  return_type: (type_annotation)? @signature.return)

;; Function Expressions / Arrow Functions
(lexical_declaration
  (variable_declarator
    name: (identifier) @definition.function
    value: [(arrow_function) (function_expression)]))

;; Enums
(enum_declaration
  name: (identifier) @definition.enum)

;; Function Calls
(call_expression
  function: [
    (identifier) @reference.call
    (member_expression property: (property_identifier) @reference.call)
  ])

;; JSX Elements (React Components)
(jsx_opening_element
  name: (identifier) @reference.component.call)

(jsx_self_closing_element
  name: (identifier) @reference.component.call)
