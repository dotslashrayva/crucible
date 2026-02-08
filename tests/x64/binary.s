	.intel_syntax noprefix
	.globl	_main                           ## -- Begin function main
_main:                                  ## @main
	push	rbp
	mov	rbp, rsp
	mov	dword ptr [rbp - 4], 0
	mov	eax, 4294967271
	pop	rbp
	ret
