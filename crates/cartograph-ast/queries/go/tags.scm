;; Go Tree-sitter Tags Query

;; Type specifications (Structs & Interfaces)
(type_spec
  name: (type_identifier) @definition.type
  type: [
    (struct_type) @definition.struct
    (interface_type) @definition.interface
  ])

;; Functions
(function_declaration
  name: (identifier) @definition.function
  parameters: (parameter_list) @signature.params)

;; Methods with receivers
(method_declaration
  receiver: (parameter_list) @scope.receiver
  name: (field_identifier) @definition.method
  parameters: (parameter_list) @signature.params)

;; Calls
(call_expression
  function: [
    (identifier) @reference.call
    (selector_expression field: (field_identifier) @reference.call)
  ])
