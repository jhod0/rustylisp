(define-macro (cond clause . rest)
  (let ((condition (car clause))
        (action (car (cdr clause)))
        (else (if rest 
                (cons 'cond rest)
                ())))
    `(if ,condition
       ,action
       ,else)))

(define-macro (assert expr)
   `(let ((evaluated ,expr))
      (if evaluated
        evaluated
        (throw-error 'assertion-error ',expr))))
