
(define (list . rest)
  rest)

(define length
  (case-lambda
    ((ls) (length ls 0))
    ((ls len)
     (if ls (length (cdr ls) (+ 1 len))
       len))))

(define (reverse ls)
  (fold-left (flip2 cons) () ls))

(let ((append-two
        (lambda (la lb)
          (if (nil? la) 
            lb
            (cons (car la)
                  (append-two (cdr la) lb))))))
  (define (append . rest)
    (fold-right append-two () rest)))
