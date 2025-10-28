(relation path (i64 i64))
(relation edge (i64 i64))

(rule ((edge x y)) ((path x y)))

(rule ((path x y) (edge y z)) ((path x z)))

(rule () ((edge 1 2)))

(rule () ((edge 2 3)))
(rule () ((edge 2 4)))

(rule () ((edge 3 5)))
(rule () ((edge 4 5)))

(run 100)

(check (path 1 5))
