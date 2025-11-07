(relation element (i64))
(relation list1 (i64))
(relation list2 (i64 i64))
(relation list3 (i64 i64 i64))
(relation list4 (i64 i64 i64 i64))

(rule ((list1 a)) ((element a)))

(rule ((list2 a b)) ((list1 a)))
(rule ((list2 a b)) ((element b)))

(rule ((list3 a b c)) ((list2 a b)))
(rule ((list3 a b c)) ((element c)))

(rule ((list4 a b c d)) ((element d)))
(rule ((list4 a b c d)) ((list3 a b c)))

(rule ((list4 a b c d)) ((list2 a b)))
(rule ((list4 a b c d)) ((list2 d c)))

(rule () ((list4 1 2 3 4)))

(run 1000)

(check (element 1))
