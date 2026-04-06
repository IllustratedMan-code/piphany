(define x 5)

(define myscript
  (file! "src/main.rs"))
(define myscript2
  (file! "src/vm.rs"))


(define metadata (read-csv "examples/test.csv"))

(set! metadata
  (~> metadata 
      (with-column "derivations" (list myscript myscript2))))
;; (define metadata (df::read-csv "examples/test.csv"))
;; (define metadata
;;   (~> metadata
;;       (df::with-column
;;        (~> (df::select-column metadata 'price)
;; 	   (column::map file!)
;; 	   )
;;        )
;;   )
;; )



(define proc1
  (process!
   name : "first-process"
   container : "ubuntu:latest"
   script : #<<''
        mkdir -p {{out}}
	cat {{(as-csv metadata "," ".csv")}} > {{out}}/csv.txt
        echo {{(+ 5 6 2 x)}} > {{out}}/result.txt
        {{myscript}} {{out}}/script-out.txt
	''
   )
  )


(define proc2
  (process!
   name : "second-process"
   time : (hours 5)
   memory : (GB 5)
   script : #<<''
   cat {{proc1}}/result.txt > {{out}}
   '')
  )


(output!
 "results/proc2-result.txt" : proc2
 "results/proc1" : proc1
 )
