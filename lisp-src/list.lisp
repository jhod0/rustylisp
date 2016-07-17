
(define length
  (case-lambda
    ((ls) (length ls 0))
    ((ls len)
     (if ls (length (cdr ls) (+ 1 len))
       len))))

(define (reverse ls)
  (fold-left (flip2 cons) () ls))
