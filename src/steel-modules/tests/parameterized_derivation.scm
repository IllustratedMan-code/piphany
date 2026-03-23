

(define (proc1 x)
  (process! (x)
   name : "parameterized_process"
   script : #<<''
   echo {{x}}
   ''
   )
  )


(proc1 5)
