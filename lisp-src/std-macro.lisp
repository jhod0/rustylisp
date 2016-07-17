;; really need quasiquote working...
(define-macro (cond clause . rest)
  (let ((condition (car clause))
        (action (car (cdr clause)))
        (else (if rest 
                (list (cons (quote cond) rest))
                (list ()))))
    ;; (quasiquote 
    ;;   (if (unquote condition)
    ;;     (unquote action)
    ;;     (unquote else)))
    (cons (quote if)
          (cons condition
                (cons action
                      else)))))
