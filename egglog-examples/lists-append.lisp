(relation element (i64))
(relation list0 ())
(relation list1 (i64))
(relation list2 (i64 i64))
(relation list3 (i64 i64 i64))
(relation list4 (i64 i64 i64 i64))

(rule ((list0) (element x)) ((list1 x)))
(rule ((list1 x) (element y)) ((list2 x y)))
(rule ((list2 x y) (element z)) ((list3 x y z)))
(rule ((list3 a b c) (element d)) ((list4 a b c d)))

(rule ((list1 a) (list1 z)) ((list2 a z)))
(rule ((list2 a b) (list1 z)) ((list3 a b z)))
(rule ((list3 a b c) (list1 z)) ((list4 a b c z)))

(rule ((list1 a) (list2 y z)) ((list3 a y z)))
(rule ((list2 a b) (list2 y z)) ((list4 a b y z)))

(rule ((list1 a) (list3 x y z)) ((list4 a x y z)))

(rule () ((element 1)))
(rule () ((element 2)))
(rule () ((element 3)))
(rule () ((element 4)))
(rule () ((list0)))

(run 1000)

(check (list4 1 2 3 4))
