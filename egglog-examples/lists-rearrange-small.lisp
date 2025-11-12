(relation element (i64))
(relation list1 (i64))


(rule ((element x)) ((list1 x)))

(rule ((list1 a)) ((element a)))


(run 100)

(check (list1 2))
