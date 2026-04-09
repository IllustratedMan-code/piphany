(define x 5)

(define script
  (file! "src/main.rs"))

(define script2
  (file! "src/vm.rs"))

(define metadata (read-csv "examples/test.csv"))

(set! metadata
  (~> metadata 
      (with-column "derivations" (list script script2))))

(define hi2 (process! name : "first3-process"
		      container : "yo" script : #<<''
		      hi''))

(define proc1
  (process!
   name : "first-process"
   container : "ubuntu:latest"
   script : #<<''
        mkdir -p {{out}}
	cat {{(as-csv metadata "," ".csv")}} > {{out}}/csv.txt
        echo {{(+ 5 6 2 x)}} > {{out}}/result.txt
        {{script}} {{out}}/script-out.txt
	''
   )
  )

(define proc1-generator (expand proc1 "*.txt" #f))


(define proc2
  (process!
   name : "second-process"
   time : (hours 5)
   memory : (GB 5)
   script : #<<''
   cat {{proc1-generator}} > {{out}}
   '')
  ) ;; proc2 is now a generator because proc1-generator is used in it


(output!
 "results/proc2-results" : proc2 ;; should generators be allowed here? or should they first be collasped
 "results/proc1" : proc1
 )
