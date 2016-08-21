(load-file "lib.lisp")

(define (repl)
  (print "> ")
  (let ((form (read))
        (res (catch-error (eval form))))
    (cond 
      ((and (error? res)
            (error-source res))
       ;; need a proper catch-error
       (begin 
         (println "error evaluating " form)
         (dump-traceback res)))
      (true
       (println res))))
  (repl))

(repl)
