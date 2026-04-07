(define x 5)

(define script
  (file! "src/main.rs"))

(define script2
  (file! "src/vm.rs"))

(define metadata (read-csv "examples/test.csv"))

(set! metadata
  (~> metadata 
      (with-column "derivations" (list script script2))))

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
