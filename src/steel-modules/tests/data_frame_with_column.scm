(define df (Dataframe
 (hash
  'a '(1 2 3)
  'b '(2 3 4)) 
 ))


(~> df
    (with-column 'c '(5 6 7)))
