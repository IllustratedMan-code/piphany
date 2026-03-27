
(define df
  (Dataframe
   (hash
    'a '(1 2 3)
    'b '(2 3 4)
  )))


(subset! df a < 2)
(subset! df a<2)
(subset! df "a<2")


