;; Python Tree-sitter Tags Query
;; Classes
(class_definition
  name: (identifier) @definition.class
  superclasses: (argument_list)? @inherits
  body: (block
    (expression_statement
      (string) @docstring)?)) @scope.class

;; Functions and Methods
(function_definition
  name: (identifier) @definition.function
  parameters: (parameters) @signature.params
  return_type: (type)? @signature.return
  body: (block
    (expression_statement
      (string) @docstring)?)) @scope.function

;; Decorated definitions
(decorated_definition
  (decorator) @decorator
  definition: [
    (function_definition)
    (class_definition)
  ])

;; Calls
(call
  function: [
    (identifier) @reference.call
    (attribute attribute: (identifier) @reference.call)
  ])

;; Imports
(import_statement
  name: (dotted_name) @reference.import)

(import_from_statement
  module_name: (dotted_name)? @reference.import.module
  name: (dotted_name) @reference.import)
