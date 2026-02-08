	.intel_syntax noprefix
	.globl	_main
_main:
	push rbp
	mov	rbp, rsp

	mov	dword ptr [rbp - 4], 0
	mov	eax, 1

	pop	rbp
	ret
