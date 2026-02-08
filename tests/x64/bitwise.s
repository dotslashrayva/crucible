	.section	__TEXT,__text,regular,pure_instructions
	.build_version macos, 15, 0	sdk_version 26, 2
	.intel_syntax noprefix
	.globl	_main                           ## -- Begin function main
	.p2align	4, 0x90
_main:                                  ## @main
## %bb.0:
	push	rbp
	mov	rbp, rsp
	mov	dword ptr [rbp - 4], 0
	mov	eax, 6
	pop	rbp
	ret
                                        ## -- End function
.subsections_via_symbols
