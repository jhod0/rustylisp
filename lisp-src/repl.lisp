(load-file "lib.lisp")

(define (eof-error? err)
  (and (symbol=? (error-type err) 'read-error)
       (symbol=? (error-value err) 'eof)))

(define (repl)
  (print "> ")
  (let ((form (catch-error (read)))
        (res (catch-error (eval form))))
    (if
      (cond 
        ;; Quit on eof error
        ((and (error? form)
              (eof-error? form))
         false)

        ;; if error evaluating, dump
        ;; a traceback
        ((and (error? res)
              (error-source res))
         ;; need a proper catch-error
         (begin 
           (println "error evaluating " form)
           (dump-traceback res)
           true))

        ;; else, print and continue
        (true
          (println res)))
      (repl)
      (string->symbol "\nGoodbye!"))))

(repl)
