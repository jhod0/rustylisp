(define (fold-left f acc ls)
  (if ls
    (fold-left f (f acc (car ls)) (cdr ls))
    acc))

(define (fold-right f acc ls)
  (if ls
    (f (car ls) 
       (fold-right f acc (cdr ls)))
    acc))

;;(let ((first-of-all
;;        (lambda (ls)
;          (if (nil? ls)
;            ()
;            (let ((hd (car (car ls)))
;                  (tl (first-of-all (cdr ls))))
;              (if (nil? tl)
;                ()
;                (cons hd tl))))
  (define (map f ls)
    (if (nil? ls)
      ()
      (let ((hd (car ls))
            (tl (cdr ls)))
        (cons (f hd) (map f tl)))))
;)

(define (flip2 f)
  (lambda (a b)
    (f b a)))
