

(define proc1
  (process!
   name : "process-1"
   script : #<<''
     echo "hi there" > {{out}}
   ''))


(define proc2
  (process!
   name : "process-2"
   script : #<<''
     cat {{proc1} > ${out}}
   ''))



