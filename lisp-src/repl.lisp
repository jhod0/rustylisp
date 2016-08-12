(load-file "lib.lisp")

(define (repl)
  (print "> ")
  (println (eval (read)))
  (repl))

(repl)
