OUT = out

.PHONY: all
all: $(OUT)/proof_system.pdf $(OUT)/ao.pdf

$(OUT)/proof_system.pdf: $(OUT)/proof_system.dot
	dot -Tpdf $(OUT)/proof_system.dot > $(OUT)/proof_system.pdf

$(OUT)/ao.pdf: $(OUT)/ao.dot
	dot -Tpdf $(OUT)/ao.dot > $(OUT)/ao.pdf
