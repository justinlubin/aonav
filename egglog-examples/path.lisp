(relation path (i64 i64))
(relation edge (i64 i64))

(rule ((edge x y)) ((path x y)))

(rule ((path x y) (edge y z)) ((path x z)))

(edge 1 2)

(edge 2 3)
(edge 2 4)

(edge 3 5)
(edge 4 5)

(check (path 1 5))
