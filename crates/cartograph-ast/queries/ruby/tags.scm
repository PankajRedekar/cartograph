;; Ruby & Rails Tree-sitter Tags Query

;; Classes
(class
  name: [
    (constant) @definition.class
    (scope_resolution) @definition.class
  ]
  superclass: (superclass)? @inherits) @scope.class

;; Modules
(module
  name: [
    (constant) @definition.module
    (scope_resolution) @definition.module
  ]) @scope.module

;; Methods
(method
  name: [
    (identifier) @definition.method
    (setter) @definition.method
  ]
  parameters: (method_parameters)? @signature.params) @scope.method

;; Singleton Methods (def self.foo)
(singleton_method
  object: [
    (self)
    (constant)
    (identifier)
  ]
  name: (identifier) @definition.method) @scope.method

;; Calls and Commands (Rails macros: has_many, belongs_to, resources, etc.)
(command
  name: (identifier) @reference.command)

(call
  receiver: [
    (identifier)
    (constant)
    (self)
  ]? @receiver
  method: (identifier) @reference.call)
