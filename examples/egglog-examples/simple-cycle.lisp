(relation A ())
(relation B ())
; (relation C ())

(rule ((A)) ((B)))
(rule ((B)) ((A)))
; (rule ((C)) ((B)))
; (rule () ((C)))

(run 100)

(check (A))
