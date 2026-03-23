(define x 5)

(define proc1
  (process!
   name : "process-1"
   script : #<<''
     mkdir -p "{{out}}"
     echo {{(+ x 1 2)}}
     ''
     ))

