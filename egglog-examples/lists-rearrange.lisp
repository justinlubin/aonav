(relation element (i64))
(relation list0 ())
(relation list1 (i64))
(relation list2 (i64 i64))
(relation list3 (i64 i64 i64))


(rule ((list0) (element x)) ((list1 x)))
(rule ((list1 x) (element y)) ((list2 x y)))
(rule ((list2 x y) (element z)) ((list3 x y z)))

(rule ((list3 a b c)) ((element c)))
(rule ((list3 a b c)) ((list2 a b)))

(rule ((list2 a b)) ((element b)))
(rule ((list2 a b)) ((element a)))

(rule () ((list2 1 2)))
(rule () ((list2 4 5)))
(rule () ((list0)))

(run 1000)

(check (list3 1 4 2))
