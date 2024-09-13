.text
.globl main
main:
	pushq %rbp
	movq %rsp, %rbp
	subq $30000, %rsp
	movsbl -29999(%rbp), %eax
	addl $8, %eax
	movb %al, -29999(%rbp)
.Lbb2:
	movsbl -29999(%rbp), %eax
	cmpl $0, %eax
	jz .Lbb4
	movsbl -30000(%rbp), %eax
	addl $9, %eax
	movb %al, -30000(%rbp)
	movsbl -29999(%rbp), %eax
	subl $1, %eax
	movb %al, -29999(%rbp)
	jmp .Lbb2
.Lbb4:
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -29999(%rbp), %eax
	addl $4, %eax
	movb %al, -29999(%rbp)
.Lbb5:
	movsbl -29999(%rbp), %eax
	cmpl $0, %eax
	jz .Lbb7
	movsbl -30000(%rbp), %eax
	addl $7, %eax
	movb %al, -30000(%rbp)
	movsbl -29999(%rbp), %eax
	subl $1, %eax
	movb %al, -29999(%rbp)
	jmp .Lbb5
.Lbb7:
	movsbl -30000(%rbp), %eax
	addl $1, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %eax
	addl $7, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %eax
	addl $3, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -29998(%rbp), %eax
	addl $6, %eax
	movb %al, -29998(%rbp)
.Lbb8:
	movsbl -29998(%rbp), %eax
	cmpl $0, %eax
	jz .Lbb10
	movsbl -29999(%rbp), %eax
	addl $7, %eax
	movb %al, -29999(%rbp)
	movsbl -29998(%rbp), %eax
	subl $1, %eax
	movb %al, -29998(%rbp)
	jmp .Lbb8
.Lbb10:
	movsbl -29999(%rbp), %eax
	addl $2, %eax
	movb %al, -29999(%rbp)
	movsbl -29999(%rbp), %edi
	callq putchar
	movsbl -29999(%rbp), %eax
	subl $12, %eax
	movb %al, -29999(%rbp)
	movsbl -29999(%rbp), %edi
	callq putchar
	movsbl -29998(%rbp), %eax
	addl $6, %eax
	movb %al, -29998(%rbp)
.Lbb11:
	movsbl -29998(%rbp), %eax
	cmpl $0, %eax
	jz .Lbb13
	movsbl -29999(%rbp), %eax
	addl $9, %eax
	movb %al, -29999(%rbp)
	movsbl -29998(%rbp), %eax
	subl $1, %eax
	movb %al, -29998(%rbp)
	jmp .Lbb11
.Lbb13:
	movsbl -29999(%rbp), %eax
	addl $1, %eax
	movb %al, -29999(%rbp)
	movsbl -29999(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %eax
	addl $3, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %eax
	subl $6, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -30000(%rbp), %eax
	subl $8, %eax
	movb %al, -30000(%rbp)
	movsbl -30000(%rbp), %edi
	callq putchar
	movsbl -29997(%rbp), %eax
	addl $4, %eax
	movb %al, -29997(%rbp)
.Lbb14:
	movsbl -29997(%rbp), %eax
	cmpl $0, %eax
	jz .Lbb16
	movsbl -29998(%rbp), %eax
	addl $8, %eax
	movb %al, -29998(%rbp)
	movsbl -29997(%rbp), %eax
	subl $1, %eax
	movb %al, -29997(%rbp)
	jmp .Lbb14
.Lbb16:
	movsbl -29998(%rbp), %eax
	addl $1, %eax
	movb %al, -29998(%rbp)
	movsbl -29998(%rbp), %edi
	callq putchar
	movl $0, %eax
	leave
	ret
.type main, @function
.size main, .-main
/* end function main */

.section .note.GNU-stack,"",@progbits
