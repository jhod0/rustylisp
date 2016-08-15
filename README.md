# About

A lisp interpreter written in Rust.

The language is based on Scheme, though is not compatible with Scheme.

# Syntax

Definitions are Scheme-like:

```
;; A function
(define (func-name arg1 arg2)
  body)

;; Alternatively...
(define func-name
  (lambda (arg1 arg2) body))

;; A variable
(define three 3)
```

Closures are created with `lambda` and `case-lambda`

```
(let ((x 1))
  (lambda (y) (+ x y)))

;; Multiple arity functions are created with case-lambda
(define foo
  (case-lambda "This is a docstring"
    ((single-arg-version) 
      'one)
    ((two-arg version) 
      'two))
;; => <named-procedure:foo>

(foo 'a)
;; => 'one
(foo 'a 'b)
;; => 'two
(doc foo)
;; => "This is a docstring"
```

Use let-bindings to limit scope:

```
(define (sum-squares a b)
  (let ((asq (* a a))
        (bsq (* b b)))
    (+ asq bsq)))
```

Symbols and linked-lists are as you would expect in a Lisp:

```
;; This is a quoted symbol
'a-symbol

(car '(1 2 3))
;; => 1
(cdr '(1 2 3))
;; => '(2 3)
```

However, there is also a lazily-evaluated cons type:

```
(cons? (lazy-cons 1 2))
;; => true

(let ((cell 
      (lazy-cons 'a
            (begin
              (println "Evaluating cdr!")
              'b))))
  (car (cdr cell)))

```

Booleans are simply the symbols `true` and `false`, which are self-evaluating.
The following values are also considered 'falsey':

* 0
* () (an empty list)
* "" (the empty string)

```
true
;; => true

(if true
    '(yay we did it)
    '(boo hoo we failed))
;; => '(yay we did it)
```

Characters are prefaced with a backslash:

```
(quote \a)
;; => \a

(string->list "A string")
;; => (\A \space \s \t \r \i \n \g)
```
