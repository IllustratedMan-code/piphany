(provide what
	 )

(define what "what")

(define the "the")

(define-syntax hi!
  (syntax-rules ()
    ((_) (print "hello world"))))

(define-syntax subset!
  (lambda (stx)
    (syntax-case stx ()
      ((_ df)
       (error "Did you forget to add a subset expression?"))
      ((_ df string)
       (begin
	 (if
	  (string? (syntax->datum #'string))
	  #'(error string)
	  (symbol->string (syntax->datum #'(string))))
	  ))
      ((_ df cond ...)
       (symbol->string (syntax->datum #'(cond ...))))
      )))

