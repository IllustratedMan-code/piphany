
(provide process!
	 file!
	 output!
	 count-nodes
	 display-nodes
	 DG::config
	 drv::display
	 DG::graph
	 read-csv
	 select
	 with-column
	 Dataframe
	 )


(require-builtin DerivationGraph as DG::)

(define read-csv DG::df::read-csv)

(define select DG::df::select)
(define subset DG::df::subset)
(define with-column DG::df::with-column)
(define Dataframe DG::df::new)

(define (process hashmap #:bindings [bindings '()])
  (let* ((script (~> (hash-get hashmap 'script)
		     (DG::ScriptString)))
	 (out-hash DG::out-hash-placeholder))
    (DG::ScriptString::set_interpolations
     script
     (map
      (lambda (x)
	(eval
	 (quasiquote
	  (let ,(append `((out ,out-hash)) bindings)
	    ,(with-input-from-string x read) ))))
      (DG::ScriptString::interpolations script)
      ))
    (set! hashmap (hash-insert hashmap 'script script)) 
    (define derivation (DG::add_derivation DG::graph
     (~> (DG::Process::new hashmap DG::config)
	 (DG::Process::as_derivation)
	 ))
      )
    derivation
    ))


;; need to report bugs here, looks like syntax case macros don't get picked up by the
;; module system correctly
;; errors are also not reported correctly
(define-syntax subset!
  (lambda (stx)
    (syntax-case stx ()
      ((_ df)
       (error "Did you forget to add a subset expression?"))
      ((_ df string)
       (begin
	 (if
	  (string? (syntax->datum #'string))
	  #`(subset df string)
	  #`(subset df #,(symbol->string (syntax->datum #'string)))
	  )))
      ((_ df cond ...)
       #`(subset df #,(symbol->string (syntax->datum #'(cond ...)))))
      )))

(define-syntax process!
  (syntax-rules ()
    [(_ (bindings) rest ...)
     (with-handler
      (lambda (err) (error err))
      (process
       (hash-helper rest ...)
       #:bindings (bindings-helper (bindings))))]
    [(_ rest ...)
     (with-handler (lambda (err) (error err))
		   (process (hash-helper rest ...)))]

  ))
	

(define-syntax bindings-helper
  (syntax-rules ()
    [(_ ())
     '()]
    [(_ (var ...))
     `((var ,var) ...)]))

(define-syntax hash-helper
  (syntax-rules (:)
    [(_ key : val)
     (hash 'key val)]
    [(_ key : val rest ...) (hash-union (hash-helper rest ...) (hash 'key val))]
    ))

(define-syntax output!
  (syntax-rules ()
  [(_ rest ...)
    (with-handler (lambda (err) (error err))
		  (DG::add_output
		   DG::graph
		   (DG::Output::new
		    (hash-helper rest ...))))]))

(define (file! path #:hashMethod [hashMethod DG::File::HashTimestamp])
  (let* ((derivation
	  (~> (DG::File::new path hashMethod)
	      (DG::File::as_derivation))))
    (DG::add_derivation DG::graph derivation)
  ))

(define (count-nodes)
  (DG::node_count DG::graph))

(define drv::display DG::Derivation::display)

(define (display-nodes)
  (DG::display_nodes DG::graph))


