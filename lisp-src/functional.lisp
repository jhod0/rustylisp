(define (fold-left f acc ls)
  (if ls
    (fold-left f (f acc (car ls)) (cdr ls))
    acc))

(define (fold-right f acc ls)
  (if ls
    (f acc (fold-right f acc (cdr ls)))
    acc))

(define (map f ls)
  (if ls
    (let ((hd (car ls))
          (tl (cdr ls)))
      (cons (f hd) (map f tl)))
    ()))

(define (flip2 f)
  (lambda (a b)
    (f b a)))
