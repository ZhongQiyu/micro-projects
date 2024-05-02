subroutine solve(A,b,x,x0,n)
implicit real*8(a-z)
integer::n,imax=200
integer::i,j,k
real*8::tol=1d-15
real*8::A(n,n),b(n),x(n),x0(n),x1(n),x2(n)

write(102,501)
501 format('Jacobi iteration',/)

x1=x0

do k=1,imax
    do i=1,n
        s=0
        do j=1,n
            if (j==i) cycle           
            s=s+A(i,j)*x1(j)      
        enddo   
        x2(i)=(b(i)-s)/A(i,i)  
    enddo
    ! the following step check if convergence is reached
    dx2=0
    do i=1,n
        dx2=dx2+(x1(i)-x2(i))**2
    enddo
    dx2=dsqrt(dx2)
    if (dx2<tol) exit
    x1=x2
    write(102,502)k,x1 ! record the iteration process
    502 format(1x,i3,<n>(1x,1pd25.15))
enddo
x=x2
end subroutine solve

program  main
implicit real*8(a-z)
integer,parameter::n=3
real*8 ::A(n,n),b(n),x(n),x0(n)

open(unit=101,file='result.txt')
open(unit=102,file='it_result.txt')

x0=(/0d0,0d0,0d0/) ! initial guess
b=(/9d0,7d0,6d0/)
A=reshape((/10,-1,0,-1,10,-4,0,-2,10/),(/3,3/))

call solve(A,b,x,x0,n) ! solve Ax=b
  
write(101,501)'x(1)','x(2)','x(3)',x
501 format('jacobi result',//,<n>(1x,a25),/,<n>(1x,1pd25.15))

end program main